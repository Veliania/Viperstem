use std::process::Command;

// Example custom build script.
fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    //println!("cargo:rerun-if-changed=src/gdt/gdt.asm");
    
    //Command::new("mkdir").args(&["objects"])
        //.status().expect("Error making objects directory");
    
    Command::new("nasm").args(&["./src/descriptors/gdt.asm", "-f elf64", "-o ./objects/gdt.o"])
        .status().expect("Error compiling gdt assembly");
}