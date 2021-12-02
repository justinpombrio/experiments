use std::fmt;
use regex::Regex;

use catalog::{Catalog, Indexed, Index};


/****************************** PUBLIC ******************************/


pub struct TokenSet<'g> {
    catalog: &'g mut Catalog<TokenData>
}

pub type Token<'g> = Indexed<'g, TokenData>;
pub type TokenId = Index<TokenData>;

impl<'g> TokenSet<'g> {

    pub fn whitespace(&mut self, name: &str, re: &str) -> TokenId {
        self.add(name, Pattern::regex(re), true, false, false)
    }
    
    pub fn identifier(name: &str, re: &str) -> TokenId {
        self.add(name, Pattern::regex(re), false, true, false)
    }

    pub fn constant(name: &str, word: &str) -> Token {
        Token::new(name, Pattern::string(word), false, true, false)
    }

    pub fn literal(name: &str, re: &str) -> Token {
        Token::new(name, Pattern::regex(re), false, true, false)
    }

    pub fn keyword(word: &str) -> Token {
        Token::new(word, Pattern::string(word), false, true, true)
    }

    pub fn punctuation(word: &str) -> Token {
        Token::new(word, Pattern::string(word), false, false, true)
    }

    pub fn magic_token(name: &str) -> Token {
        Token::new(name, Pattern::None, false, false, true)
    }

    fn add(&mut self, name: &str, pattern: Pattern,
           ignore: bool, req_space: bool, is_part: bool) -> TokenId {
        let token = TokenData::new(name, pattern, ignore, req_space, is_part);
        self.catalog.add(token)
    }
}


/****************************** PRIVATE ******************************/


#[derive(Clone, Debug)]
pub enum Pattern {
    String(String),
    Regex(Regex),
    None
}

impl Pattern {
    pub fn string(s: &str) -> Pattern {
        Pattern::String(s.to_string())
    }
    pub fn regex(re: &str) -> Pattern {
        match Regex::new(re) {
            Ok(regex) => Pattern::Regex(regex),
            Err(err) =>
                panic!("Lexer: bad regular expression {}", err)
        }
    }
}

#[derive(Clone)]
pub struct TokenData {
    pub name: String,
    pub pattern: Pattern,
    pub ignore: bool,
    pub req_space: bool,
    pub is_part: bool
}

impl TokenData {
    fn new(name: &str, pattern: Pattern, ignore: bool, req_space: bool, is_part: bool) -> TokenData {
        TokenData{
            name:      name.to_string(),
            pattern:   pattern,
            ignore:    ignore,
            req_space: req_space,
            is_part: is_part
        }
    }
}

impl fmt::Display for TokenData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl fmt::Debug for TokenData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
