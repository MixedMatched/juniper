use std::{io::Write, process::Command};

fn main() {
    println!("cargo::rerun-if-changed=../../../lean");
    let mut rebuild = Command::new("lake");

    rebuild.arg("lean").arg("JuniperLean.lean");
    rebuild.current_dir("../../lean/");

    let output = rebuild.output().expect("Building JupiterLean failed");

    std::io::stdout().write_all(&output.stdout).unwrap();
}
