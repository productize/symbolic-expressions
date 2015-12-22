// (c) 2015 Joost Yervante Damad <joost@damad.be>

extern crate rust_sexp;

fn main() {
    let s = rust_sexp::parse_file("examples/SILABS_EFM32_QFN24.kicad_mod");
    println!("{}", s);
}
