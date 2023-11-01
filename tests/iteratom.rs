// (c) 2017 Productize SPRL <joost@productize.be>

extern crate generic_symbolic_expressions;

use generic_symbolic_expressions::iteratom::*;
use generic_symbolic_expressions::Sexp;
use generic_symbolic_expressions::SexpError;

struct Qq(i64);

impl FromSexp for Qq {
    fn from_sexp(s: &Sexp) -> Result<Self, SexpError> {
        let i = s.named_value_i("d")?;
        Ok(Qq(i))
    }
}

fn test_int() -> Result<(), SexpError> {
    let s = "(a (b c) (d 42))";
    let s = generic_symbolic_expressions::parser::parse_str(s)?;
    let mut i = IterAtom::new(&s, "a")?;
    let c = i.s_in_list("b")?;
    assert_eq!(&c, "c");
    let e: Qq = i.t("d")?;
    assert_eq!(e.0, 42);
    Ok(())
}

#[test]
fn test_iteratom() {
    test_int().unwrap();
}
