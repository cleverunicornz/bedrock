//! C2 YAML-syntax scanning: anchors, aliases, and merge keys are forbidden
//! (SPINE §4 C2).
//!
//! Enforcement MUST be at YAML-syntax level, never text regex: the seed's
//! floor vertices legitimately contain markdown emphasis (`*can*`,
//! `*consumption*`) inside block scalars, which a text regex on `*` would
//! false-positive on.
//!
//! serde_norway *resolves* anchors internally and exposes no syntax API, so
//! this module scans the raw YAML token stream with `unsafe-libyaml-norway`
//! (the exact tokenizer serde_norway may be built on). The scanner yields
//! first-class `YAML_ANCHOR_TOKEN`/`YAML_ALIAS_TOKEN` tokens with line marks;
//! block-scalar content is a single scalar token and can never contain them.
//! Merge keys are a scalar `<<` in mapping-key position.

#[allow(clippy::unsafe_removed_from_name)]
use unsafe_libyaml_norway as sys;

/// One forbidden YAML construct found in a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YamlViolation {
    /// An `&name` anchor definition.
    Anchor { line: u32, name: String },
    /// A `*name` alias use.
    Alias { line: u32, name: String },
    /// A `<<` merge key.
    MergeKey { line: u32 },
    /// The YAML failed to tokenize at all.
    Tokens { line: u32, message: String },
}

fn mark_line(mark: &sys::yaml_mark_t) -> u32 {
    mark.line as u32 + 1
}

/// Raw-scan `input` for anchor/alias/merge-key tokens.
///
/// `input` must be valid UTF-8 (callers parse as UTF-8 elsewhere); scanned
/// bytes are copied to owned strings before the token buffer is freed.
pub fn scan_forbidden(input: &[u8]) -> Vec<YamlViolation> {
    // The parser must not move after initialization (it holds a pointer to
    // the input buffer), so it lives pinned on the heap alongside the
    // borrowed input. Parser + input share one allocation lifetime.
    // The parser is uninit; yaml_parser_initialize fills it. It must not
    // move after init (it points at the input buffer) and the input must
    // outlive the scan, so both live on the heap through this function.
    let holder = Box::new(std::mem::MaybeUninit::<sys::yaml_parser_t>::uninit());
    // SAFETY: `holder` keeps both the parser memory and (through a manual
    // borrow that lives as long as this function) the input alive; we never
    // move the parser after init and always delete tokens before freeing.
    let parser: *mut sys::yaml_parser_t = holder.as_ptr().cast_mut();
    let _pin_guard = holder;

    unsafe {
        if sys::yaml_parser_initialize(parser).fail {
            return vec![YamlViolation::Tokens {
                line: 1,
                message: "libyaml parser initialization failed".to_string(),
            }];
        }
        sys::yaml_parser_set_encoding(parser, sys::YAML_UTF8_ENCODING);
        sys::yaml_parser_set_input_string(parser, input.as_ptr(), input.len() as u64);
    }

    let mut out = Vec::new();
    // Merge-key detection: `<<` must appear as the scalar of a mapping key.
    let mut pending_key = false;

    loop {
        let mut token = std::mem::MaybeUninit::<sys::yaml_token_t>::uninit();
        // SAFETY: as above; token buffer is owned by libyaml and deleted below.
        let ok = unsafe { sys::yaml_parser_scan(parser, token.as_mut_ptr()) };
        let mut token = if ok.fail {
            // SAFETY: read error state out of the live, initialized parser.
            let line = unsafe { (&*parser).problem_mark.line as u32 + 1 };
            // The failed scan left no initialized token to free.
            out.push(YamlViolation::Tokens {
                line,
                message: "YAML scanner error".to_string(),
            });
            break;
        } else {
            // SAFETY: yaml_parser_scan succeeded → token is initialized.
            unsafe { token.assume_init() }
        };

        let line = mark_line(&token.start_mark);
        match token.type_ {
            sys::YAML_KEY_TOKEN => pending_key = true,
            sys::YAML_VALUE_TOKEN
            | sys::YAML_BLOCK_ENTRY_TOKEN
            | sys::YAML_FLOW_ENTRY_TOKEN
            | sys::YAML_BLOCK_END_TOKEN
            | sys::YAML_FLOW_SEQUENCE_END_TOKEN
            | sys::YAML_FLOW_MAPPING_END_TOKEN => {
                pending_key = false;
            }
            sys::YAML_ANCHOR_TOKEN => {
                let name = unsafe { read_cstr(token.data.anchor.value) };
                out.push(YamlViolation::Anchor { line, name });
            }
            sys::YAML_ALIAS_TOKEN => {
                let name = unsafe { read_cstr(token.data.alias.value) };
                out.push(YamlViolation::Alias { line, name });
            }
            sys::YAML_SCALAR_TOKEN => {
                let (value, style) = unsafe {
                    (
                        read_bytes(token.data.scalar.value, token.data.scalar.length as usize),
                        token.data.scalar.style,
                    )
                };
                // Merge keys are `<<` in key position. Style is irrelevant:
                // a quoted "<<" key is still a merge attempt.
                if pending_key && value == "<<" {
                    out.push(YamlViolation::MergeKey { line });
                }
                pending_key = false;
                let _ = style;
            }
            sys::YAML_STREAM_END_TOKEN => {
                // SAFETY: free the stream-end token, then stop.
                unsafe { sys::yaml_token_delete(&mut token) };
                break;
            }
            _ => {}
        }
        // SAFETY: free the token buffer libyaml allocated for this token.
        unsafe { sys::yaml_token_delete(&mut token) };
    }

    // SAFETY: no further use of the parser.
    unsafe { sys::yaml_parser_delete(parser) };
    out
}

/// Read a NUL-terminated C string into an owned String (raw pointer to UTF-8).
unsafe fn read_cstr(ptr: *mut u8) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let mut len = 0usize;
    // SAFETY: the caller guarantees `ptr` points at a NUL-terminated buffer.
    while unsafe { *ptr.add(len) } != 0 {
        len += 1;
    }
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    String::from_utf8_lossy(bytes).into_owned()
}

/// Read `len` bytes from a raw pointer into owned Vec.
unsafe fn read_bytes(ptr: *mut u8, len: usize) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    String::from_utf8_lossy(bytes).into_owned()
}
