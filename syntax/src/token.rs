use crate::string_pool::StringPool;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'p> {
    Ident(&'p str),
    Number(f64),
    Punct(&'p str),
}

pub struct Lexer<'s, 'p> {
    input: Peekable<Chars<'s>>,
    string_pool: &'p StringPool,
}

impl<'s, 'p> Lexer<'s, 'p> {
    pub fn new(input: &'s str, pool: &'p StringPool) -> Self {
        Lexer {
            input: input.chars().peekable(),
            string_pool: pool,
        }
    }

    fn skip_whitespaces(&mut self) {
        while let Some(&c) = self.input.peek() {
            if !c.is_whitespace() {
                break;
            }
            self.input.next();
        }
    }

    pub fn next_token(&mut self) -> Option<Token<'p>> {
        self.skip_whitespaces();
        let cur_char = match self.input.peek() {
            None => return None,
            Some(ch) => ch,
        };
        match cur_char {
            'a'..='z' | 'A'..='Z' => {
                let mut s = String::new();
                while let Some(&c) = self.input.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        s.push(c);
                        self.input.next();
                    } else {
                        break;
                    }
                }
                Some(Token::Ident(self.string_pool.get(&s)))
            }
            '0'..='9' => {
                let mut dot_meeted = false;
                let mut s = String::new();
                while let Some(&c) = self.input.peek() {
                    if c.is_digit(10) || (c == '.' && !dot_meeted) {
                        if c == '.' {
                            dot_meeted = true;
                        }
                        s.push(c);
                        self.input.next();
                    } else {
                        break;
                    }
                }
                Some(Token::Number(s.parse().unwrap()))
            }
            _ => {
                let mut s = String::new();
                while let Some(&c) = self.input.peek() {
                    if c.is_whitespace() {
                        break;
                    } else {
                        s.push(c);
                        self.input.next();
                    }
                }
                Some(Token::Punct(self.string_pool.get(&s)))
            }
        }
    }
}

impl<'p> Token<'p> {
    pub fn tokenlize<'s>(input: &'s str, string_pool: &'p StringPool) -> Vec<Token<'p>> {
        let mut lexer = Lexer::new(input, string_pool);
        let mut tokens = Vec::new();
        while let Some(token) = lexer.next_token() {
            tokens.push(token);
        }
        tokens
    }
}
