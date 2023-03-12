#![feature(trace_macros)]
use macros::bnf;
use syntax::*;
use string_pool::*;
use bnf::*;

struct Identifier {

}

struct Number {

}

trace_macros!(true);
bnf! {
    only: Start => Exper;
    add: Exper => Exper '+' Exper;
    sub: Exper => Exper '-' Exper;
    number: Exper => Number;
}
trace_macros!(false);

fn construct_ast(parse_tree: ParseTree) -> Start {
    todo!()
}

#[test]
fn test_bnf() {
    let pool = StringPool::new();
    println!("{:#?}", get_bnf(&pool));
}
