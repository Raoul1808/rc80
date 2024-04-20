# rc80

> *Rusty Chip-8 Emulator*

A CHIP-8 emulator written in Rust, for practice and because I was bored.

The emulation part and the frontend are kept separate.
The frontend uses `eframe` to display additional debugging info while running.

## Development Resources

- [eframe](https://github.com/emilk/egui/tree/master/crates/eframe)
- [CHIP-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#3.0)
- [CHIP-8 Variant Opcode Table](https://chip8.gulrak.net/)
- [CHIP-8 Test Suite](https://github.com/Timendus/chip8-test-suite)

## Building

Pre-requisites:

- A rust toolchain

Steps:

1. Clone this repo
2. Run `cargo r -r`
3. Profit

## License

This project is licensed under the [MIT License](LICENSE)
