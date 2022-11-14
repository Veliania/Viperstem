use std::process::Command;

fn main() {
    Command::new("nasm").args(&["./src/descriptors/gdt.asm", "-f elf64", "-o ./objects/gdt.o"])
        .status().expect("Error compiling gdt assembly");

    for arg in [
    "--gc-sections",
    "--script=.cargo/kernel.ld",
    "./objects/gdt.o"
    ] {
        println!("cargo:rustc-link-arg={arg}");
    }
}