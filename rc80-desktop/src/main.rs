use std::io;

use rc80_core::System;

fn main() {
    let mut sys = System::default();
    let bytes = include_bytes!("/home/mew/Downloads/1-chip8-logo.ch8");
    sys.load(bytes);
    loop {
        sys.step();
        io::stdin().read_line(&mut String::new()).unwrap();
    }
}
