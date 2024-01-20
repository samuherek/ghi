
#[derive(Debug, PartialEq)]
enum Token {
    LSq,
    RSq,
    LAr, 
    RAr,
    Flag,
    Str(String),
    Illegal,
    Eof,
}

struct CmdLexer<'a > {
    input: &'a str, 
    position: usize,
    read_position: usize,
    ch: char
}

impl<'a> CmdLexer<'a> {
    fn new(input: &'a str) -> Self {
        let mut lexer = Self {
            input,
            position: 0,
            read_position: 0,
            ch: '0'
        };
        lexer.read_char();
        return lexer;
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '0';
        } else {
            self.ch = self.input
                .chars()
                .nth(self.read_position)
                .expect("read position to access the correct char");
        }

        self.position = self.read_position;
        self.read_position += 1;
    }

    fn read_str(&mut self) -> String {
        let pos = self.position;
        while is_str_letter(self.ch) {
            self.read_char();
        }
        return self.input[pos..self.position].to_string();
    }

    fn skip_whitespace(&mut self) {
        while self.ch == ' ' {
            self.read_char();
        }
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace();        

        let token = match self.ch {
            '[' => Token::LSq,
            ']' => Token::RSq,
            '<' => Token::LAr,
            '>' => Token::RAr,
            '-' => Token::Flag,
            'a'..='z' | 'A'..='Z' => {
                let value = self.read_str(); 
                return Token::Str(value);
            },
            '0' => Token::Eof,
            _ => Token::Illegal
        };
        self.read_char();
        return token;
    }
}

fn is_str_letter(input: char) -> bool {
    match input {
        'a'..='z' | 'A'..='Z' | '-' | '_' => true,
        _ => false
    }
}

fn split_cmd(input: &str) -> (&str, &str) {
    if input.contains(' ') {
        input.split_once(' ').expect("input to be splittable on space")
    } else {
        (input, &"")
    }
}

//"suspend-client [-t <target-client>]"
pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut lexer = CmdLexer::new(&input);
    let mut to_next = true;

    while to_next {
        let token = lexer.next_token(); 
        if token == Token::Eof {
            to_next = false;
        }
        tokens.push(token);
    }

    return tokens;
}


#[cfg(test)]
mod tests {
    #[test]
    fn split_cmd() {
        let value = "choose-client [-t <target-session>]";
        let (cmd, _) = super::split_cmd(&value);

        assert_eq!(cmd, "choose-client");

        let value = "choose-client";
        let (cmd, _) = super::split_cmd(&value);

        assert_eq!(cmd, "choose-client");
    }

    #[test]
    fn next_token() {
        let value = "choose-client []";
        let (_, input) = super::split_cmd(&value);
        let mut lexer = super::CmdLexer::new(&input);

        let token = lexer.next_token();
        assert_eq!(token, super::Token::LSq);

        let token = lexer.next_token();
        assert_eq!(token, super::Token::RSq);

    }
    
    #[test]
    fn lex() {
        let input = "[]<>f";
        let exp = vec![
            super::Token::LSq,
            super::Token::RSq,
            super::Token::LAr,
            super::Token::RAr,
            super::Token::Str(String::from("f")),
            super::Token::Eof,
        ];

        let result = super::lex(&input);

        assert_eq!(result, exp);
    }

    #[test]
    fn flag() {
        let input = "[]<>-fa[";
        let exp = vec![
            super::Token::LSq,
            super::Token::RSq,
            super::Token::LAr,
            super::Token::RAr,
            super::Token::Flag,
            super::Token::Str(String::from("fa")),
            super::Token::LSq,
            super::Token::Eof,
        ];

        let result = super::lex(&input);

        assert_eq!(result, exp);
    }

    #[test]
    fn command() {
        let input = "[-t <target-session>]";
        let exp = vec![
            super::Token::LSq,
            super::Token::Flag,
            super::Token::Str(String::from("t")),
            super::Token::LAr,
            super::Token::Str(String::from("target-session")),
            super::Token::RAr,
            super::Token::RSq,
            super::Token::Eof,
        ];

        let result = super::lex(&input);

        assert_eq!(result, exp);
    }
}
