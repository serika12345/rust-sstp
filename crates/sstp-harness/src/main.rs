mod command;
mod presentation;

use std::process::ExitCode;

fn main() -> ExitCode {
    match command::parse(std::env::args().skip(1))
        .and_then(command::execute)
        .map(presentation::render)
    {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("エラー: {error}");
            ExitCode::FAILURE
        }
    }
}
