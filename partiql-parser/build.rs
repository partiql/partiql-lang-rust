use std::env::current_dir;
use std::io;

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=src/parse/partiql.lalrpop");

    let grammar_dir = current_dir()?.join("src").join("parse");
    lalrpop::Configuration::new()
        .set_in_dir(grammar_dir)
        .process_current_dir()
        .expect("lalrpop process");

    Ok(())
}
