fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match bedrock::cli::parse(&args) {
        Ok(cmd) => match bedrock::cli::run(cmd) {
            Ok(code) => code,
            Err(e) => {
                eprintln!("{e}");
                1
            }
        },
        Err(e) => {
            eprintln!("{e}");
            1
        }
    };
    std::process::exit(code);
}
