use mayak::{execute, parse_args, print_help, Action, VERSION};
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    match parse_args(args) {
        Ok(Action::Run(config)) => {
            if let Err(err) = execute(config) {
                eprintln!("mayak: error: {}", err);
                process::exit(1);
            }
        }
        Ok(Action::Version) => {
            println!("mayak v{}", VERSION);
        }
        Ok(Action::Help) => {
            print_help("mayak");
        }
        Err(err) => {
            eprintln!("mayak: {}", err);
            process::exit(2);
        }
    }
}
