use crate::{compilation_state::label_generator::LabelGenerator, data_type::type_token::TypeInfo, number_literal::NumberLiteral, string_literal::StringLiteral};

use super::{token::Token, punctuator::Punctuator};

fn is_keyword(text: &str) -> bool {
    let possible_keywords = vec!["break", "case", "continue", "default", "do", "else", "enum", "for", "goto", "if", "return", "sizeof", "struct", "switch", "typedef", "union", "while"];
    
    return possible_keywords.contains(&text);
}

pub struct Lexer{
    data: String,
    next_to_eat: usize,//index of next character to consume
    string_label_generator: LabelGenerator
}

impl Lexer {
    fn peek(&self) -> Option<char> {
        self.data.chars().nth(self.next_to_eat)
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
    pub fn new(sanitised_file: &str) -> Lexer{
        Lexer{
            data: sanitised_file.to_string(),
            next_to_eat:0,
            string_label_generator: LabelGenerator::new()
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

    fn consume_str(&mut self) -> Token {
        let mut result_text = String::new();
        loop {
            let curr_char = self.consume().unwrap();
            let next_char = self.peek().unwrap();
            result_text += &curr_char.to_string();

            if next_char == '"' && curr_char != '\\' {
                //non escaped speech mark
                self.consume();//consume last speech mark
                result_text += "\"";//add the close speech mark
                return Token::STRING(StringLiteral::try_new(&result_text, &mut self.string_label_generator).unwrap());
            }
        }
    }

    fn consume_punctuation(&mut self) -> Token{

        //special case for elipsis ...
        if self.data[self.next_to_eat..].starts_with("...") {
            for _ in 0..3 {
                self.consume().unwrap();//consume dot of elipsis
            }
            return Token::PUNCTUATOR(Punctuator::ELIPSIS);
        }

        let curr_char = self.consume().unwrap();

        let next_char = match self.peek(){
            Some(x) => x,
            None => ' '//run out of chars, use whitespace
        };

        let possible_double_char = format!("{}{}", curr_char, next_char);//in case I want to match "==" or similar

        if let Some(punc) = Punctuator::try_new(&possible_double_char) {
            //double character punctuation "=="
            self.consume().unwrap();//consume the token, as I have used it
            Token::PUNCTUATOR(punc)
        } else {
            Token::PUNCTUATOR(Punctuator::try_new(&curr_char.to_string()).unwrap())
        }
    }

    fn consume_number(&mut self) -> Token {
        let mut letters = String::new();

        while let Some(c) = self.peek() {
            if !("0123456789.".contains(c)) {
                break;
            }
            letters.push(c);
            self.consume();
        }

        if Some('f') == self.peek() {
            panic!("float suffix not implemented");
        }
        if Some('e') == self.peek() {
            panic!("standard form is not implemented");
        }

        assert!(letters.len() > 0);
        
        return Token::NUMBER(NumberLiteral::try_new(&letters).unwrap());
    }

    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        match self.peek()? {
            c if c.is_numeric() => Some(self.consume_number()),
            c if c.is_alphabetic() || c == '_' => Some(self.consume_generic_text()),
            c if "(){}[];,+-*/=&%><.".contains(c) => Some(self.consume_punctuation()),
            '"' => Some(self.consume_str()),
            _ => None//maybe panic here?
        }
    }
}