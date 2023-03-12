extern crate proc_macro;
use std::iter::Peekable;
use proc_macro::{TokenStream, TokenTree};
use proc_macro2::{Ident, Span};
use quote::quote;
use syntax::{bnf::*, string_pool::StringPool};

fn parse<'p, T>(tokens: &mut Peekable<T>, pool: &'p StringPool) -> Option<Rule<'p>>
where
    T: Iterator<Item = TokenTree>,
{
    let name = match tokens.next() {
        Some(TokenTree::Ident(ident)) => ident.to_string(),
        _ => return None,
    };
    match tokens.next() {
        Some(TokenTree::Punct(punct)) if punct.as_char() == ':' => (),
        _ => return None,
    };
    let head = match tokens.next() {
        Some(TokenTree::Ident(ident)) => NonTerminal {
            name: pool.get(&ident.to_string()),
        },
        _ => return None,
    };
    match tokens.next() {
        Some(TokenTree::Punct(punct)) if punct.as_char() == '=' => (),
        _ => return None,
    };
    match tokens.next() {
        Some(TokenTree::Punct(punct)) if punct.as_char() == '>' => (),
        _ => return None,
    };
    let mut body = Vec::<Symbol>::new();
    while let Some(token_tree) = tokens.peek() {
        match token_tree {
            TokenTree::Ident(ident) => {
                body.push(Symbol::NonTerminal(NonTerminal {
                    name: pool.get(&ident.to_string()),
                }));
                tokens.next();
            }
            TokenTree::Literal(literal) => {
                body.push(Symbol::Terminal(Terminal {
                    name: pool.get(&literal.to_string()),
                }));
                tokens.next();
            }
            TokenTree::Punct(punct) if punct.as_char() == ';' => {
                tokens.next();
                break;
            }
            _ => {}
        }
    }
    Some(Rule {
        name: pool.get(&name),
        head,
        body,
    })
}

#[proc_macro]
pub fn bnf(item: TokenStream) -> TokenStream {
    let mut rules = Vec::<Rule>::new();
    let mut tokens = item.into_iter().peekable();
    let pool = StringPool::new();
    while let Some(rule) = parse(&mut tokens, &pool) {
        rules.push(rule);
    }
    let bnf = BNF {
        start: NonTerminal::new(pool.get("start")),
        rules,
    };
    let bnf_proxy = BNFProxy::new(&bnf);
    let structs = bnf_proxy
        .rules_iter()
        .map(|(head, rules)| {
            let head_ident = Ident::new(head.name, Span::call_site());
            let variants = rules
                .iter()
                .map(|(variant_name, rule)| {
                    let fields = rule
                        .body
                        .iter()
                        .filter(|symbol| match symbol {
                            Symbol::NonTerminal(_) => true,
                            Symbol::Terminal(_) => false,
                        })
                        .map(|symbol| match symbol {
                            Symbol::NonTerminal(nonterm) => {
                                Ident::new(&nonterm.name, Span::call_site())
                            }
                            Symbol::Terminal(_) => panic!(),
                        })
                        .collect::<Vec<_>>();
                    let variant_ident = Ident::new(&variant_name, Span::call_site());
                    quote! {#variant_ident(#(Box<#fields>),*)}
                })
                .collect::<Vec<_>>();
            quote! {
                enum #head_ident {
                    #(#variants),*
                }
            }
        })
        .collect::<Vec<_>>();
    let strs = pool.iter().collect::<Vec<_>>();
    let pool_get_fn = quote! {
        pub fn get_pool() -> StringPool {
            let ret = StringPool::new();
            [#(#strs),*].iter().for_each(|s| { ret.get(&**s); });
            ret
        }
    };
    let result = quote! {
        #(#structs)*
        #pool_get_fn
    };
    result.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
