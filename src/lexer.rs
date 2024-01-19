
#[derive(Debug, PartialEq)]
enum Token {
    LSquare,
    RSquare,
    LArrow, 
    RArrow,
    Illegal,
    Eof
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
            ch: '\n'
        };
        lexer.read_char();
        return lexer;
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\n';
        } else {
            self.ch = self.input
                .chars()
                .nth(self.read_position)
                .expect("read position to access the correct char");
        }

        self.position = self.read_position;
        self.read_position += 1;
    }

    fn next_token(&mut self) -> Token {
        let token =match self.ch {
            '[' => Token::LSquare,
            _ => Token::Illegal
        };
        self.read_char();
        return token;
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
pub fn lex(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut lexer = CmdLexer::new(input);
    lexer.next_token(); 

    println!("{}", {input});

    tokens.push("hellow".to_string());
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
        let value = "choose-client [-t <target-session>]";
        let (_, input) = super::split_cmd(&value);
        let mut lexer = super::CmdLexer::new(&input);

        let token = lexer.next_token();
        assert_eq!(token, super::Token::LSquare);

        let token = lexer.next_token();
        assert_eq!(token, super::Token::LSquare);

    }
}
