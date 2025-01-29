use super::token::Token;

pub struct Lexer{
    data: String,
    next_to_eat: usize//index of next character to consume
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
    pub fn new(file_data: &str) -> Lexer{
        Lexer{
            data: file_data.to_string(),
            next_to_eat:0
        }
    }

    fn consume_identifier_or_keyword(&mut self) -> Token {
        let mut letters = String::new();

        while let Some(c) = self.peek() {
            if !c.is_alphanumeric() && c != '_' {
                break;
            }
            letters.push(c);
        }

        assert!(letters.len() > 0);

        Token::KWORDORIDENT(letters)
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



    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();

        match self.peek()? {
            c if c.is_alphanumeric() || c == '_' => Some(self.consume_identifier_or_keyword()),
            c if "(){};".contains(c) => Some(self.consume_punctuation()),
            _ => None
        }
    }
}