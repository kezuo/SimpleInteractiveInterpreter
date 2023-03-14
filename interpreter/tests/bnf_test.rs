#![feature(trace_macros)]
use bnf::*;
use macros::bnf;
use string_pool::*;
use syntax::*;
use token::*;

#[derive(Debug)]
struct Identifier {}

#[derive(Debug)]
struct Number {}

trace_macros!(true);
bnf! {
    only: Start => Exper;
    add: Exper => Exper '+' Exper;
    sub: Exper => Exper '-' Exper;
    number: Exper => Number;
}
trace_macros!(false);

fn construct_Number<'b, 'p, 'tree, 'token, T>(
    pool: &'p StringPool,
    parse_tree: &'tree Vec<ParseTree>,
    tokens: &mut T,
) -> Box<Number>
where
    'p: 'token,
    T: Iterator<Item = &'token Token<'p>>,
{
    assert!(match tokens.next().unwrap() {
        Token::Ident(_) => false,
        Token::Number(_) => true,
        Token::Punct(_) => false,
    });
    Box::new(Number {})
}

fn construct_Identifier<'b, 'p, 'tree, 'token, T>(
    pool: &'p StringPool,
    parse_tree: &'tree Vec<ParseTree>,
    tokens: &mut T,
) -> Box<Identifier>
where
    'p: 'token,
    T: Iterator<Item = &'token Token<'p>>,
{
    assert!(match tokens.next().unwrap() {
        Token::Ident(_) => false,
        Token::Number(_) => true,
        Token::Punct(_) => false,
    });
    Box::new(Identifier {})
}

fn construct_Exper_number<'b, 'p, 'tree, 'token, T>(
    pool: &'p StringPool,
    parse_tree: &'tree Vec<ParseTree>,
    tokens: &mut T,
) -> Box<Exper>
where
    'p: 'token,
    T: Iterator<Item = &'token Token<'p>>,
{
    let number0 = construct_Number(pool, parse_tree, tokens);
    Box::new(Exper::number(number0))
}

fn construct_Exper_add<'b, 'p, 'tree, 'token, T>(
    pool: &'p StringPool,
    parse_tree: &'tree Vec<ParseTree>,
    tokens: &mut T,
) -> Box<Exper>
where
    'p: 'token,
    T: Iterator<Item = &'token Token<'p>>,
{
    let exper0 = construct_Exper(pool, &parse_tree[0], tokens);
    let exper1 = construct_Exper(pool, &parse_tree[1], tokens);
    Box::new(Exper::add(exper0, exper1))
}

fn construct_Exper_sub<'b, 'p, 'tree, 'token, T>(
    pool: &'p StringPool,
    parse_tree: &'tree Vec<ParseTree>,
    tokens: &mut T,
) -> Box<Exper>
where
    'p: 'token,
    T: Iterator<Item = &'token Token<'p>>,
{
    let exper0 = construct_Exper(pool, &parse_tree[0], tokens);
    let exper1 = construct_Exper(pool, &parse_tree[1], tokens);
    Box::new(Exper::sub(exper0, exper1))
}

fn construct_Exper<'b, 'p, 'tree, 'token, T>(
    pool: &'p StringPool,
    parse_tree: &'tree ParseTree,
    tokens: &mut T,
) -> Box<Exper>
where
    'p: 'token,
    T: Iterator<Item = &'token Token<'p>>,
{
    let rule = parse_tree.rule.name;
    if pool.str_eq(rule, "add") {
        return construct_Exper_add(pool, &parse_tree.sub_trees, tokens);
    }
    if pool.str_eq(rule, "sub") {
        return construct_Exper_sub(pool, &parse_tree.sub_trees, tokens);
    }
    if pool.str_eq(rule, "number") {
        return construct_Exper_number(pool, &parse_tree.sub_trees, tokens);
    }
    panic!("tokens not matching rule")
}

fn construct_Start_only<'b, 'p, 'tree, 'token, T>(
    pool: &'p StringPool,
    parse_tree: &'tree Vec<ParseTree>,
    tokens: &mut T,
) -> Box<Start>
where
    'p: 'token,
    T: Iterator<Item = &'token Token<'p>>,
{
    let exper0 = construct_Exper(pool, &parse_tree[0], tokens);
    Box::new(Start::only(exper0))
}

fn construct_Start<'b, 'p, 'tree, 'token, T>(
    pool: &'p StringPool,
    parse_tree: &'tree ParseTree,
    tokens: &mut T,
) -> Box<Start>
where
    'p: 'token,
    T: Iterator<Item = &'token Token<'p>>,
{
    let rule = parse_tree.rule.name;
    if pool.str_eq(rule, "only") {
        return construct_Start_only(pool, &parse_tree.sub_trees, tokens);
    }
    panic!("tokens not matching rule")
}

#[test]
fn test_bnf() {
    let pool = StringPool::new();
    let bnf = get_bnf(&pool);
    let tokens = Token::tokenlize("a + b", &pool);
    let bnf_proxy = BNFProxy::new(&bnf);
    let parse_tree = parse(tokens.iter(), &bnf_proxy, &pool).unwrap().1;
    let ast = construct_Start(&pool, &parse_tree, &mut tokens.iter());
    println!("{:#?}", ast);
}
