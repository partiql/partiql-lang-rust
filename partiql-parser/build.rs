//extern crate lalrpop;

use std::env::current_dir;
use std::io;

fn main() -> io::Result<()> {
    let grammer_dir = current_dir()?.join("src").join("lalr");
    lalrpop::Configuration::new()
        .set_in_dir(grammer_dir.clone())
        .process_current_dir()
        .expect("lalrpop process");

    Ok(())
}
