# deimOS
***An operating system written in Rust and named after the moon of Mars, the rusty planet.***

To get started, simply run:
```bash
make emulate
```
or inside './deimos/'
```bash
cargo run --release
```
This should launch a qemu session with the operating system (after compilation).

Dependancies:
- rust toolchain
- qemu-full
- gnu make (soft dependancy)

Cargo Dependancies (See './deimos/Cargo.toml' for versions and features)
-  *bootloader* 
-  *volatile* 
-  *lazy_static*
-  *spin*
-  *x86_64*
-  *uart_16550*
-  *pic8259*
-  *pc-keyboard*

*This project was inspired by, and uses code from the `Wiriting an OS in Rust` blog by Philipp Opperman.*
[https://os.phil-opp.com/]
[https://github.com/phil-opp/blog_os]

