use mayak::{execute, parse_args, print_help, Action, VERSION};
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    match parse_args(args) {
        Ok(Action::Run(config)) => {
            if let Err(err) = execute(config) {
                eprintln!("mk: error: {}", err);
                process::exit(1);
            }
        }
        Ok(Action::Version) => {
            println!("mk v{}", VERSION);
        }
        Ok(Action::Help) => {
            print_help("mk");
        }
        Err(err) => {
            eprintln!("mk: {}", err);
            process::exit(2);
        }
    }
}
