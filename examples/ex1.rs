// (c) 2015-2016 Productize SPRL <joost@productize.be>

extern crate generic_symbolic_expressions;

fn main() {
    let s =
        generic_symbolic_expressions::parser::parse_file("examples/SILABS_EFM32_QFN24.kicad_mod")
            .unwrap();
    println!("{}", s);
}
