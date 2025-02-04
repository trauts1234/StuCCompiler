use crate::{number_literal::NumberLiteral, operator::Operator, type_info::TypeInfo};

use super::{remove_comments::remove_comments, token::Token};

fn is_keyword(text: &str) -> bool {
    let possible_keywords = vec!["break", "case", "continue", "default", "do", "else", "enum", "for", "goto", "if", "return", "sizeof", "struct", "switch", "typedef", "union", "while", "_Bool"];
    
    return possible_keywords.contains(&text);
}

pub struct Lexer{
    data: String,
    next_to_eat: usize//index of next character to consume
}

impl Lexer {
    fn peek(&self) -> Option<char> {
        self.data.chars().nth(self.next_to_eat)
    }
    fn peek_after_next(&self) -> Option<char> {
        self.data.chars().nth(self.next_to_eat+1)
    }
    fn consume(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.next_to_eat += 1;
        }
        return c;
    }
    fn skip_whitespace(&mut self) {
        while let Some(_) = self.peek().filter(|x| " \n".contains(*x)) {//while there are tokens, and they are space or newline
            self.consume();
        }
    }
}

impl Lexer {
    /**
     * take in raw data from a file, to generate a tokenizing lexer
     * note: automatically deals with \ \n and comments etc.
     */
    pub fn new(file_data: &str) -> Lexer{
        let comment_adjusted_data = remove_comments(file_data);
        Lexer{
            data: comment_adjusted_data,
            next_to_eat:0
        }
    }

    fn consume_generic_text(&mut self) -> Token {
        let mut letters = String::new();

        while let Some(c) = self.peek() {
            if !c.is_alphanumeric() && c != '_' {
                break;
            }
            letters.push(c);
            self.consume();
        }

        assert!(letters.len() > 0);

        //try to match with a known keyword
        if is_keyword(&letters) {
            Token::KEYWORD(letters)
        } else if let Some(type_data) = TypeInfo::try_new(&letters) {
            Token::TYPESPECIFIER(type_data)
        } else {
            Token::IDENTIFIER(letters)
        }
    }

    fn consume_punctuation(&mut self) -> Token{
        let c = self.consume().unwrap();

        if c == '#' && self.peek() == Some('#') {
            self.consume();//remove the second hashtag
            //infamous double hashtag
            return Token::PUNCTUATION("##".to_owned());
        }

        //all other punctuation is single-character
        Token::PUNCTUATION(c.to_string())
    }

    fn consume_number(&mut self) -> Token {
        let mut letters = String::new();

        if Some('-') == self.peek() {
            letters.push('-');
            self.consume();//consume initial negative sign
        }

        while let Some(c) = self.peek() {
            if !("0123456789.".contains(c)) {
                break;
            }
            letters.push(c);
            self.consume();
        }

        if Some('f') == self.peek() {
            panic!("float suffix not implemented")
        }

        assert!(letters.len() > 0);
        
        return Token::NUMBER(NumberLiteral::try_new(&letters).unwrap());
    }

    fn consume_operator(&mut self) -> Token {
        let c = self.consume().unwrap();

        if c == '+' && self.peek() == Some('+') {
            //this is actually ++, not +
            panic!("unary operators not implemented")
        }
        
        Token::OPERATOR(Operator::try_new(&c.to_string()).unwrap())
    }



    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        match self.peek()? {
            c if c.is_numeric() || //starts with number
                (c == '-' && self.peek_after_next().is_some_and(|x| x.is_numeric()))//starts with -(number)
                    => Some(self.consume_number()),
            c if c.is_alphabetic() || c == '_' => Some(self.consume_generic_text()),
            c if "(){};".contains(c) => Some(self.consume_punctuation()),
            c if "+".contains(c) => Some(self.consume_operator()),
            _ => None
        }
    }
}