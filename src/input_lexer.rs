
///
/// TODO: implement a delimiter "--"
/// ---
/// example: cargo run -- arg
///
/// TODO: implement an explicit flag input
/// ---
/// example: command --option=23
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    FlagShort(String),
    FlagCombo(String),
    FlagLong(String),
    Str(String),
    Int(usize) // TODO: not implemented yet
}

pub struct InputCmdLexer<'a > {
    input: &'a str, 
    position: usize,
    read_position: usize,
    ch: Option<char>
}

impl<'a> InputCmdLexer<'a> {
    pub fn compile(input: &'a str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut lexer = Self {
            input,
            position: 0,
            read_position: 0,
            ch: None
        };
        lexer.read_char();

        while let Some(token) = lexer.next_token() {
            tokens.push(token);
        }

        tokens
    }

    fn read_char(&mut self) {
        self.ch = self.input.chars().nth(self.read_position);
        self.position = self.read_position;
        self.read_position += 1;
    }

    fn peak_char(&self) -> Option<char> {
        self.input.chars().nth(self.read_position)
    }

    fn read_str(&mut self) -> String {
        let pos = self.position;
        while self.peak_char().is_some_and(|x| x != ' ')  {
            self.read_char();
        }
        return self.input[pos..=self.position].to_string();
    }

    fn read_input(&mut self, quote: char) -> String {
        let pos = self.position;
        while self.peak_char().is_some_and(|x| x != quote) {
            self.read_char(); 
        }
        return self.input[pos..=self.position].to_string();
    }

    fn skip_whitespace(&mut self) {
        while self.ch == Some(' ')  {
            self.read_char();
        }
    }
    // fn has_next_token(&self) -> bool {
    //     self.ch.is_some() 
    // }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();        

        let res = if let Some(token) = self.ch {
            match token {
                '"' => {
                    self.read_char();
                    let value = self.read_input('"');
                    self.read_char();
                    Some(Token::Str(value))
                },
                '\'' => {
                    self.read_char();
                    let value = self.read_input('\'');
                    self.read_char();
                    Some(Token::Str(value))
                }
                '-' => {
                    if self.peak_char() == Some('-') {
                        self.read_char();
                        self.read_char();
                        let value = self.read_str(); 
                        Some(Token::FlagLong(value))
                    } else {
                        self.read_char();
                        let value = self.read_str(); 
                        Some(Token::FlagShort(value))
                    }
                },
                _ => {
                    let value = self.read_str(); 
                    // TODO: might need to use "peak" instead of "read"
                    // so that we don't have to "return" which is inconsistent
                    // with the rest of the arms.
                    Some(Token::Str(value))
                }
            }
        } else {
            None
        };
        self.read_char();
        res
    }
}

// TODO: implement the possibility to have numbers inside a string 
// that does not start with a number.
// Currently, it breaks, because we use '0' as an EOF enum
fn is_str_letter(input: Option<char>) -> bool {
    if let Some(input) = input {
        match input {
            'a'..='z' | 'A'..='Z' | '-' | '_' | '0'..='9'  => true,
            _ => false
        }
    } else {
        false
    }
}



#[cfg(test)]
mod tests {
    use super::*;
     
    #[test]
    fn lex() {
        let input = "display-message -p -t client-0 'Hello, World!'";
        let exp = vec![
            Token::Str(String::from("display-message")),
            Token::FlagShort(String::from("p")),
            Token::FlagShort(String::from("t")),
            Token::Str(String::from("client-0")),
            Token::Str(String::from("Hello, World!")),
        ];
        let result = InputCmdLexer::compile(&input);
        assert_eq!(result, exp);
    }
     
    #[test]
    fn cmd1() {
        let input = "capture-pane -t pane-1";
        let exp = vec![
            Token::Str(String::from("capture-pane")),
            Token::FlagShort(String::from("t")),
            Token::Str(String::from("pane-1")),
        ];
        let result = InputCmdLexer::compile(&input);
        assert_eq!(result, exp);
    }
     
    #[test]
    fn cmd2() {
        let input = "save-buffer -b 1 /path/to/file.txt";
        let exp = vec![
            Token::Str(String::from("save-buffer")),
            Token::FlagShort(String::from("b")),
            Token::Str(String::from("1")),
            Token::Str(String::from("/path/to/file.txt")),
        ];
        let result = InputCmdLexer::compile(&input);
        assert_eq!(result, exp);
    }

    #[test]
    fn cmd3() {
        let input = "paste-buffer -t pane-2 -b 5";
        let exp = vec![
            Token::Str(String::from("paste-buffer")),
            Token::FlagShort(String::from("t")),
            Token::Str(String::from("pane-2")),
            Token::FlagShort(String::from("b")),
            Token::Str(String::from("5")),
        ];
        let result = InputCmdLexer::compile(&input);
        assert_eq!(result, exp);
    }

    #[test]
    fn cmd4() {
        let input = "if-shell -b -t pane-3 \"test -f myfile.txt\" \"echo File exists\" \"echo File does not exist\"";
        let exp = vec![
            Token::Str(String::from("if-shell")),
            Token::FlagShort(String::from("b")),
            Token::FlagShort(String::from("t")),
            Token::Str(String::from("pane-3")),
            Token::Str(String::from("test -f myfile.txt")),
            Token::Str(String::from("echo File exists")),
            Token::Str(String::from("echo File does not exist")),
        ];
        let result = InputCmdLexer::compile(&input);
        assert_eq!(result, exp);
    }
}
