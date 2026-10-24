//! Desktop entry point; all setup lives in the library so it can be tested.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--cli") {
        let rest: Vec<String> = args.into_iter().filter(|a| a != "--cli").collect();
        std::process::exit(etchwise_lib::cli::run(&rest));
    }
    etchwise_lib::run();
}
