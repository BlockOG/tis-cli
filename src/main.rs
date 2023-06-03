mod direction;
mod instruction;
mod node;
mod number;
mod parse_tis;
mod position;
mod register;
mod tis;

use std::{env::args, fs::read_to_string};

use parse_tis::parse;
use tis::TIS;

fn main() {
    let mut args = args();
    args.next();
    let code = read_to_string(args.next().expect("No path provided")).unwrap();

    let mut tis = TIS::new();
    parse(&mut tis, code);

    loop {
        tis.tick();
    }
}
