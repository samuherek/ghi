
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    LSq,
    RSq,
    LAr, 
    RAr,
    FlagShort(String),
    FlagLong(String),
    Multiple,
    Or,
    Str(String),
    Illegal(char),
}

pub struct CmdLexer<'a > {
    input: &'a str, 
    position: usize,
    read_position: usize,
    ch: Option<char>
}

impl<'a> CmdLexer<'a> {
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
        while is_str_letter(self.peak_char()) {
            self.read_char();
        }
        return self.input[pos..=self.position].to_string();
    }

    fn read_flag(&mut self) -> String {
        let pos = self.position;
        while is_flag_letter(self.peak_char()) {
            self.read_char();
        }
        return self.input[pos..=self.position].to_string();
    }

    fn skip_whitespace(&mut self) {
        while self.ch == Some(' ') {
            self.read_char();
        }
    }

    fn consume_dots(&mut self) {
        while self.peak_char() == Some('.') {
            self.read_char();
        }
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();        

        if let Some(token) = self.ch {
            let token = match token {
                '[' => Some(Token::LSq),
                ']' => Some(Token::RSq),
                '<' => Some(Token::LAr),
                '>' => Some(Token::RAr),
                '-' => {
                    if self.peak_char() == Some('-') {
                        self.read_char();
                        self.read_char();
                        Some(Token::FlagLong(self.read_flag()))
                    } else {
                        self.read_char();
                        Some(Token::FlagShort(self.read_flag()))
                    }
                },
                '.' => {
                    self.consume_dots();
                    Some(Token::Multiple)
                },
                '|' => Some(Token::Or),
                'a'..='z' | 'A'..='Z' => Some(Token::Str(self.read_str())),
                _ => Some(Token::Illegal(token))
            };
            self.read_char();
            token
        } else {
            None
        }
    }
}

fn is_flag_letter( input: Option<char>) -> bool {
    if let Some(input) = input {
        match input {
            'a'..='z' | 'A'..='Z' | '-'  => true,
            _ => false
        }
    } else {
        false
    }
}

// TODO: implement the possibility to have numbers inside a string 
// that does not start with a number.
// Currently, it breaks, because we use '0' as an EOF enum
fn is_str_letter(input: Option<char>) -> bool {
    if let Some(input) = input {
        match input {
            'a'..='z' | 'A'..='Z' | '-' | '_'  => true,
            _ => false
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn just_once_letter() {
        let input = "a";
        let exp = vec![
            super::Token::Str(String::from("a")),
        ];
        let result = super::CmdLexer::compile(&input);
        assert_eq!(result, exp);
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
        ];
        let result = super::CmdLexer::compile(&input);
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
            super::Token::FlagShort(String::from("fa")),
            super::Token::LSq,
        ];
        let result = super::CmdLexer::compile(&input);
        assert_eq!(result, exp);
    }

    #[test]
    fn command() {
        let input = "[-t <target-session>]";
        let exp = vec![
            super::Token::LSq,
            super::Token::FlagShort(String::from("t")),
            super::Token::LAr,
            super::Token::Str(String::from("target-session")),
            super::Token::RAr,
            super::Token::RSq,
        ];
        let result = super::CmdLexer::compile(&input);
        assert_eq!(result, exp);
    }

    #[test]
    fn cmd1() {
        let input = "[-t <current-name>] <new-name>";
        let exp = vec![
            super::Token::LSq,
            super::Token::FlagShort(String::from("t")),
            super::Token::LAr,
            super::Token::Str(String::from("current-name")),
            super::Token::RAr,
            super::Token::RSq,
            super::Token::LAr,
            super::Token::Str(String::from("new-name")),
            super::Token::RAr, 
        ];
        let result = super::CmdLexer::compile(&input);
        assert_eq!(result, exp);
    }

    #[test]
    fn cmd2() {
        let input = "[-c <target-client>] [-t <target-session>]";
        let exp = vec![
            super::Token::LSq,
            super::Token::FlagShort(String::from("c")),
            super::Token::LAr,
            super::Token::Str(String::from("target-client")),
            super::Token::RAr,
            super::Token::RSq,
            super::Token::LSq,
            super::Token::FlagShort(String::from("t")),
            super::Token::LAr,
            super::Token::Str(String::from("target-session")),
            super::Token::RAr,
            super::Token::RSq,
        ];
        let result = super::CmdLexer::compile(&input);
        assert_eq!(result, exp);
    }

    #[test]
    fn cmd3() {
        let input = "[-b] [-t <target-pane>] <shell-command>";
        let exp = vec![
            super::Token::LSq,
            super::Token::FlagShort(String::from("b")),
            super::Token::RSq,
            super::Token::LSq,
            super::Token::FlagShort(String::from("t")),
            super::Token::LAr,
            super::Token::Str(String::from("target-pane")),
            super::Token::RAr,
            super::Token::RSq,
            super::Token::LAr,
            super::Token::Str(String::from("shell-command")),
            super::Token::RAr,
        ];
        let result = super::CmdLexer::compile(&input);
        assert_eq!(result, exp);
    }

    #[test]
    fn cmd4() {
        let input = "[-s <session-name>] [-n <window-name>] [command]";
        let exp = vec![
            super::Token::LSq,
            super::Token::FlagShort(String::from("s")),
            super::Token::LAr,
            super::Token::Str(String::from("session-name")),
            super::Token::RAr,
            super::Token::RSq,
            super::Token::LSq,
            super::Token::FlagShort(String::from("n")),
            super::Token::LAr,
            super::Token::Str(String::from("window-name")),
            super::Token::RAr,
            super::Token::RSq,
            super::Token::LSq,
            super::Token::Str(String::from("command")),
            super::Token::RSq,
        ];
        let result = super::CmdLexer::compile(&input);
        assert_eq!(result, exp);
    }

    #[test]
    fn cmd5() {
        let input = "[-b] [-t <target-pane>] <shell-command> <command> [else-command]";
        let exp = vec![
            super::Token::LSq,
            super::Token::FlagShort(String::from("b")),
            super::Token::RSq,
            super::Token::LSq,
            super::Token::FlagShort(String::from("t")),
            super::Token::LAr,
            super::Token::Str(String::from("target-pane")),
            super::Token::RAr,
            super::Token::RSq,
            super::Token::LAr,
            super::Token::Str(String::from("shell-command")),
            super::Token::RAr,
            super::Token::LAr,
            super::Token::Str(String::from("command")),
            super::Token::RAr,
            super::Token::LSq,
            super::Token::Str(String::from("else-command")),
            super::Token::RSq,
        ];
        let result = super::CmdLexer::compile(&input);
        assert_eq!(result, exp);
    }

    #[test]
    fn flag_with_or() {
        let input = "[-D | -U] [-t <target-pane>]";
        let exp = vec![
            super::Token::LSq,
            super::Token::FlagShort(String::from("D")),
            super::Token::Or,
            super::Token::FlagShort(String::from("U")),
            super::Token::RSq,
            super::Token::LSq,
            super::Token::FlagShort(String::from("t")),
            super::Token::LAr,
            super::Token::Str(String::from("target-pane")),
            super::Token::RAr,
            super::Token::RSq,
        ];
        let result = super::CmdLexer::compile(&input);
        assert_eq!(result, exp);
    }

    #[test]
    fn long_argument() {
        let input = "[-b <branch>] [--depth <depth>] <repository> [<directory>]";
        let exp = vec![
            super::Token::LSq,
            super::Token::FlagShort(String::from("b")),
            super::Token::LAr,
            super::Token::Str(String::from("branch")),
            super::Token::RAr,
            super::Token::RSq,
            super::Token::LSq,
            super::Token::FlagLong(String::from("depth")),
            super::Token::LAr,
            super::Token::Str(String::from("depth")),
            super::Token::RAr,
            super::Token::RSq,
            super::Token::LAr,
            super::Token::Str(String::from("repository")),
            super::Token::RAr,
            super::Token::LSq,
            super::Token::LAr,
            super::Token::Str(String::from("directory")),
            super::Token::RAr,
            super::Token::RSq,
        ];
        let result = super::CmdLexer::compile(&input);
        assert_eq!(result, exp);
    }

    #[test]
    fn two_long_flags() {
        let input = "[--hard | --soft] <commit>";
        let exp = vec![
            super::Token::LSq,
            super::Token::FlagLong(String::from("hard")),
            super::Token::Or,
            super::Token::FlagLong(String::from("soft")),
            super::Token::RSq,
            super::Token::LAr,
            super::Token::Str(String::from("commit")),
            super::Token::RAr,
        ];
        let result = super::CmdLexer::compile(&input);
        assert_eq!(result, exp);
    }
    
    #[test]
    fn flag_with_dash() {
        let input = "[--hard-one] <commit>";
        let exp = vec![
            super::Token::LSq,
            super::Token::FlagLong(String::from("hard-one")),
            super::Token::RSq,
            super::Token::LAr,
            super::Token::Str(String::from("commit")),
            super::Token::RAr,
        ];
        let result = super::CmdLexer::compile(&input);
        assert_eq!(result, exp);
    }
    
   #[test]
   fn command_and_subcommand() {
       let input = "git reset [--hard | --soft] <commit>";
       let exp = vec![
           super::Token::Str(String::from("git")),
           super::Token::Str(String::from("reset")),
           super::Token::LSq,
           super::Token::FlagLong(String::from("hard")),
           super::Token::Or,
           super::Token::FlagLong(String::from("soft")),
           super::Token::RSq,
           super::Token::LAr,
           super::Token::Str(String::from("commit")),
           super::Token::RAr,
       ];
       let result = super::CmdLexer::compile(&input);
       assert_eq!(result, exp);
   }

   #[test]
   fn multiple_dot_inputs() {
       let input = "git add [<file>... | <directory>...]";
       let exp = vec![
           super::Token::Str(String::from("git")),
           super::Token::Str(String::from("add")),
           super::Token::LSq,
           super::Token::LAr,
           super::Token::Str(String::from("file")),
           super::Token::RAr,
           super::Token::Multiple,
           super::Token::Or,
           super::Token::LAr,
           super::Token::Str(String::from("directory")),
           super::Token::RAr,
           super::Token::Multiple,
           super::Token::RSq,
       ];
       let result = super::CmdLexer::compile(&input);
       assert_eq!(result, exp);
   }
}
