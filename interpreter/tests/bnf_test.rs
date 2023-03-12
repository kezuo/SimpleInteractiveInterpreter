use macros::bnf;
use string_pool::*;
use syntax::*;

struct Identifier {

}

struct Number {

}

bnf! {
    only: start => exper;
    add: exper => exper '+' exper;
    sub: exper => exper '-' exper;
    number: exper => Number;
}

#[test]
fn test_bnf() {
    println!("{:?}", get_pool().iter().collect::<Vec<_>>());
}
