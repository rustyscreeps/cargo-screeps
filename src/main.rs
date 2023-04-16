mod build;
mod config;
mod copy;
mod orientation;
mod run;
mod setup;
mod upload;

fn main() {
    if let Err(e) = run::run() {
        eprintln!("error: {:?}", e);
        std::process::exit(1);
    }
}
