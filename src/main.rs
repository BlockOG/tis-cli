mod direction;
mod instruction;
mod node;
mod number;
mod parse_tis;
mod position;
mod register;
mod tis;
mod utils;

use std::env::args;

use parse_tis::parse;
use tis::TIS;

fn main() {
    if let Err(Some(e)) = run_code() {
        eprintln!("{}", e);
    }
}

fn run_code() -> Result<(), Option<String>> {
    let mut args = args();
    args.next();

    let mut tis = TIS::new();
    parse(&mut tis, args.next().ok_or("No path provided".to_owned())?)?;

    loop {
        tis.tick();
    }
}
