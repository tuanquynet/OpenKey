mod app;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match app::run_cli(&args) {
        Ok(output) => {
            if !output.is_empty() {
                println!("{}", output);
            }
        }
        Err(err) => {
            eprintln!("error: {}", err);
            std::process::exit(1);
        }
    }
}
