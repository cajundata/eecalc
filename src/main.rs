fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match eecalc::run(&args) {
        Ok(output) => println!("{output}"),
        Err(err) => {
            eprintln!("error: {err}");
            std::process::exit(err.exit_code());
        }
    }
}
