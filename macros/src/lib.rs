extern crate proc_macro;
use proc_macro::{TokenStream, TokenTree};
use proc_macro2::{Ident, Span};
use quote::quote;
use std::iter::Peekable;
use syntax::{bnf::*, string_pool::StringPool};

trait ToTokenStream {
    fn to_token_stream(&self) -> proc_macro2::TokenStream;
}

impl<'p> ToTokenStream for Terminal<'p> {
    fn to_token_stream(&self) -> proc_macro2::TokenStream {
        let name = self.name;
        quote! {
            Terminal::new(pool.get(#name))
        }
    }
}

impl<'p> ToTokenStream for NonTerminal<'p> {
    fn to_token_stream(&self) -> proc_macro2::TokenStream {
        let name = self.name;
        quote! {
            NonTerminal::new(pool.get(#name))
        }
    }
}

impl<'p> ToTokenStream for Symbol<'p> {
    fn to_token_stream(&self) -> proc_macro2::TokenStream {
        match self {
            Symbol::NonTerminal(v) => {
                let nonterm = v.to_token_stream();
                quote! {
                    Symbol::NonTerminal(#nonterm)
                }
            }
            Symbol::Terminal(v) => {
                let term = v.to_token_stream();
                quote! {
                    Symbol::Terminal(#term)
                }
            }
        }
    }
}

impl<'p> ToTokenStream for Rule<'p> {
    fn to_token_stream(&self) -> proc_macro2::TokenStream {
        let name = self.name;
        let head = self.head.to_token_stream();
        let body = self
            .body
            .iter()
            .map(|symbol| symbol.to_token_stream())
            .collect::<Vec<_>>();
        quote! {
            Rule {
                name: #name,
                head: #head,
                body: vec![#(#body),*],
            }
        }
    }
}

impl<'p> ToTokenStream for BNF<'p> {
    fn to_token_stream(&self) -> proc_macro2::TokenStream {
        let start = self.start.to_token_stream();
        let rules = self
            .rules
            .iter()
            .map(|rule| rule.to_token_stream())
            .collect::<Vec<_>>();
        quote! {
            BNF {
                start: #start,
                rules: vec![#(#rules),*],
            }
        }
    }
}

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
        start: NonTerminal::new(pool.get("Start")),
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
                #[derive(Debug)]
                enum #head_ident {
                    #(#variants),*
                }
            }
        })
        .collect::<Vec<_>>();
    let create_bnf = bnf.to_token_stream();
    let get_bnf_fn = quote! {
        pub fn get_bnf<'p>(pool: &'p StringPool) -> BNF<'p> {
            let bnf = #create_bnf;
            bnf
        }
    };
    let result = quote! {
        #(#structs)*
        #get_bnf_fn
    };
    result.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        struct test<'b> {
            s: &'b str,
        }
        let t = test { s: "hello world" };
        println!("{:?}", quote! {#{t.s}});
    }
}
