// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let mut args = std::env::args().skip(1);
    let expr = args.next();
    if args.next().is_some() {
        std::process::exit(1);
    }
    let piped = !std::io::IsTerminal::is_terminal(&std::io::stdin());
    match expr {
        None if !piped => hataclip_lib::run(),
        Some(expr) => std::process::exit(hataclip_lib::run_cli(&expr, piped)),
        None => std::process::exit(1),
    }
}
