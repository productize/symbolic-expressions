use std::fmt;
use std::fs::File;

enum Atom {
  S(String),
  Q(String),
  I(i64),
  F(f64),
}

enum Sexp {
  Atom(Atom),
  List(Vec<Sexp>),
}

impl fmt::Display for Atom {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      Atom::S(ref s) => write!(f, "{}", s),
      Atom::Q(ref s) => write!(f, "\"{}\"", s),
      Atom::I(i)     => write!(f, "{}", i),
      Atom::F(d)     => write!(f, "{}", d),
    }
  }
}

impl fmt::Display for Sexp {
  fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    match *self {
      Sexp::Atom(ref a) => write!(f, "{}", a),
      Sexp::List(ref xs) => {
        try!(write!(f, "("));
        for (i, x) in xs.iter().enumerate() {
          let s = if i == 0 { "" } else { " " };
          try!(write!(f, "{}{}", s, x));
        }
        write!(f, ")")
      },
    }
  }
}

fn parse_list(vec: &Vec<char>, mut pos: usize) -> (usize, Sexp) {
    println!("list");
    pos += 1;
    let mut l: Vec<Sexp> = Vec::new();
    loop {
        pos = match vec[pos] {
            ')' => {
                break;
            }
            _ => {
                let (pos, s) = parse_sexp(vec, pos);
                l.push(s);
                pos
            }
        }
    }
    println!("Found list");
    (pos, Sexp::List(l))
}

fn parse_quoted_string(vec: &Vec<char>, mut pos: usize) -> (usize, Atom) {
    println!("qstring");
    pos += 1;
    let mut s = String::new();
    loop {
        match vec[pos] {
            '"' => break,
            x => s.push(x)
        }
        pos += 1;
    }
    println!("Found quoted string {}", s);
    (pos+1, Atom::Q(s))
}

fn parse_string(vec: &Vec<char>, mut pos: usize) -> (usize, Atom) {
    println!("string");
    let mut s = String::new();
    loop {
        match vec[pos] {
            ' ' | ')' => break,
            '"' => panic!("quote in unquoted string"),
            x => s.push(x)
        }
        pos += 1;
    }
    println!("Found string {}", s);
    (pos, Atom::S(s))
}

fn parse_number(vec: &Vec<char>, mut pos: usize) -> (usize, Atom) {
    println!("number");
    let mut s = String::new();
    loop {
        match vec[pos] {
            ' ' | ')' => break,
            '0' ... '9' | '.' | '-' => s.push(vec[pos]),
            _ => panic!("unexpected char in number"),
        }
        pos += 1;
    }
    println!("Found number {}", s);
    let s2: &str = &s[..];
    if s.contains('.') {
        (pos, Atom::F(f64::from_str(s2).unwrap()))
    } else {
        (pos, Atom::I(i64::from_str(s2).unwrap()))
    }
}

fn parse_atom(vec: &Vec<char>, pos: usize) -> (usize, Sexp) {
    println!("atom");
    let (pos, a) = match vec[pos] {
        '"' => {
            parse_quoted_string(vec, pos)
        }
        '0' ... '9' | '.' | '-' => {
            parse_number(vec, pos)
        }
        _ => {
            parse_string(vec, pos)
        }
    };
    (pos, Sexp::Atom(a))
}


fn parse_sexp(vec: &Vec<char>, pos: usize) -> (usize, Sexp) {
    let mut pos = pos;
    loop {
        match vec[pos] {
            '(' => {
                return parse_list(vec, pos)
            }
            ' ' | '\r' | '\t' | '\n' => {
                pos += 1;
            }
            _ => {
                return parse_atom(vec, pos)
            }
        }
    }
}

fn parse(data: &str) -> Sexp {
    let vec: Vec<char> = data.chars().collect();
    let (_, res) = parse_sexp(&vec, 0);
    res
}

fn main() {
    let mut f = try!(File::open("data/SILABS_EFM32_QFN24.kicad_mod"));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));
    let res = parse(s);
    println!("res: {}", res)
}
