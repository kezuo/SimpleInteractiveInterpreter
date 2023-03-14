use crate::string_pool::StringPool;
use crate::token::*;
use core::hash::Hash;
use core::slice::Iter;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
pub struct NonTerminal<'p> {
    pub name: &'p str,
}

impl<'p> NonTerminal<'p> {
    pub fn new(name: &'p str) -> Self {
        Self { name }
    }
}

impl<'p> Hash for NonTerminal<'p> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Terminal<'p> {
    pub name: &'p str,
}

impl<'p> Terminal<'p> {
    pub fn new(name: &'p str) -> Self {
        Self { name }
    }
}

impl<'p> Terminal<'p> {
    pub fn match_token(&self, token: &Token<'p>, pool: &'p StringPool) -> bool {
        match token {
            Token::Ident(s) | Token::Punct(s) => std::ptr::eq(self.name, *s),
            Token::Number(_) => std::ptr::eq(self.name, pool.get("num")),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Symbol<'p> {
    NonTerminal(NonTerminal<'p>),
    Terminal(Terminal<'p>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Rule<'p> {
    pub name: &'p str,
    pub head: NonTerminal<'p>,
    pub body: Vec<Symbol<'p>>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct BNF<'p> {
    pub start: NonTerminal<'p>,
    pub rules: Vec<Rule<'p>>,
}

pub struct BNFProxy<'b, 'p> {
    start: &'b NonTerminal<'p>,
    non_term2rules: HashMap<&'b NonTerminal<'p>, HashMap<&'p str, &'b Rule<'p>>>,
}

impl<'b, 'p> BNFProxy<'b, 'p> {
    pub fn new(bnf: &'b BNF<'p>) -> Self {
        let mut non_term2rules =
            HashMap::<&'b NonTerminal<'p>, HashMap<&'p str, &'b Rule<'p>>>::new();
        bnf.rules.iter().for_each(|rule| {
            if let Some(str2rule) = non_term2rules.get_mut(&rule.head) {
                str2rule.insert(&rule.name, &rule);
            } else {
                let mut str2rule = HashMap::<&'p str, &'b Rule>::new();
                str2rule.insert(&rule.name, &rule);
                non_term2rules.insert(&rule.head, str2rule);
            }
        });
        Self {
            start: &bnf.start,
            non_term2rules,
        }
    }

    pub fn has_nonterm(&self, name: &'static str) -> bool {
        self.non_term2rules.keys().any(|key| key.name == name)
    }

    pub fn rules(&self, head: &'b NonTerminal<'p>) -> Option<&HashMap<&'b str, &'b Rule<'p>>> {
        self.non_term2rules.get(head)
    }

    pub fn rules_iter(
        &self,
    ) -> impl Iterator<Item = (&&'b NonTerminal<'p>, &HashMap<&'p str, &'b Rule<'p>>)> {
        self.non_term2rules.iter()
    }

    pub fn start(&self) -> &NonTerminal {
        self.start
    }
}

#[derive(PartialEq)]
pub struct ParseTree<'b, 'p> {
    pub rule: &'b Rule<'p>,
    pub sub_trees: Vec<ParseTree<'b, 'p>>,
}

fn parse_rule<'b, 't, 'p>(
    rule: &'b Rule<'p>,
    mut token_iter: Iter<'t, Token<'p>>,
    bnf: &'_ BNFProxy<'b, 'p>,
    pool: &'p StringPool,
) -> Option<(Iter<'t, Token<'p>>, ParseTree<'b, 'p>)> {
    let mut sub_trees = Vec::new();

    for symbol in rule.body.iter() {
        match symbol {
            Symbol::NonTerminal(head) => match bnf.rules(head) {
                Some(rules) => {
                    let ret = rules
                        .iter()
                        .find_map(|(_, rule)| parse_rule(rule, token_iter.clone(), bnf, pool));
                    match ret {
                        None => return None,
                        Some((left, sub_tree)) => {
                            token_iter = left;
                            sub_trees.push(sub_tree);
                        }
                    }
                }
                None => panic!(),
            },
            Symbol::Terminal(terminal) => match token_iter.next() {
                Some(t) if terminal.match_token(t, pool) => (),
                _ => return None,
            },
        }
    }
    Some((token_iter, ParseTree { rule, sub_trees }))
}

fn parse_head<'b, 't, 'p>(
    head: &'b NonTerminal<'p>,
    token_iter: Iter<'t, Token<'p>>,
    bnf: &'_ BNFProxy<'b, 'p>,
    pool: &'p StringPool,
) -> Option<(Iter<'t, Token<'p>>, ParseTree<'b, 'p>)> {
    match bnf.rules(head) {
        None => None,
        Some(rules) => rules
            .iter()
            .find_map(|(_, rule)| parse_rule(rule, token_iter.clone(), bnf, pool)),
    }
}

pub fn parse<'b, 't, 'p>(
    token_iter: Iter<'t, Token<'p>>,
    bnf: &'_ BNFProxy<'b, 'p>,
    pool: &'p StringPool,
) -> Option<(Iter<'t, Token<'p>>, ParseTree<'b, 'p>)> {
    parse_head(bnf.start, token_iter, bnf, pool)
}
