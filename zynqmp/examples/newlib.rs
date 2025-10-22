use std::process;

use zynqmp as _; // Ensure that the crate is linked into the binary

fn main() {
    let mut string = String::new();
    string.push_str("Hello");
    string.push(' ');
    string.push_str("Newlib");
    string.push('!');

    println!("{string}");

    process::exit(0);
}
