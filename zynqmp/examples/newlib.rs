use std::process;

zynqmp::entry!(main);

fn main() -> ! {
    unsafe { zynqmp::uart::uart0().initialize() }

    let mut string = String::new();
    string.push_str("Hello");
    string.push(' ');
    string.push_str("Newlib");
    string.push('!');

    println!("{string}");

    process::exit(0);
}
