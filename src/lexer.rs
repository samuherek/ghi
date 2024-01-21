
#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    LSq,
    RSq,
    LAr, 
    RAr,
    FlagShort,
    FlagLong,
    Multiple,
    Or,
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
        self.ch = self.input.chars().nth(self.read_position).unwrap_or('0');
        self.position = self.read_position;
        self.read_position += 1;
    }

    fn peak_char(&self) -> char {
        self.input.chars().nth(self.read_position).unwrap_or('0')
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

    fn consume_dots(&mut self) {
        while self.ch == '.' {
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
            '-' => {
                if self.peak_char() == '-' {
                    self.read_char();
                    Token::FlagLong
                } else {
                    Token::FlagShort
                }
            },
            '.' => {
                self.consume_dots();
                return Token::Multiple;
            },
            '|' => Token::Or,
            'a'..='z' | 'A'..='Z' => {
                let value = self.read_str(); 
                // TODO: might need to use "peak" instead of "read"
                // so that we don't have to "return" which is inconsistent
                // with the rest of the arms.
                return Token::Str(value)
            },
            '0' => Token::Eof,
            _ => Token::Illegal
        };
        self.read_char();
        return token;
    }
}

// TODO: implement the possibility to have numbers inside a string 
// that does not start with a number.
// Currently, it breaks, because we use '0' as an EOF enum
fn is_str_letter(input: char) -> bool {
    match input {
        'a'..='z' | 'A'..='Z' | '-' | '_'  => true,
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
            super::Token::FlagShort,
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
            super::Token::FlagShort,
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

    #[test]
    fn cmd1() {
        let input = "[-t <current-name>] <new-name>";
        let exp = vec![
            super::Token::LSq,
            super::Token::FlagShort,
            super::Token::Str(String::from("t")),
            super::Token::LAr,
            super::Token::Str(String::from("current-name")),
            super::Token::RAr,
            super::Token::RSq,
            super::Token::LAr,
            super::Token::Str(String::from("new-name")),
            super::Token::RAr, 
            super::Token::Eof,
        ];
        let result = super::lex(&input);
        assert_eq!(result, exp);
    }

    #[test]
    fn cmd2() {
        let input = "[-c <target-client>] [-t <target-session>]";
        let exp = vec![
            super::Token::LSq,
            super::Token::FlagShort,
            super::Token::Str(String::from("c")),
            super::Token::LAr,
            super::Token::Str(String::from("target-client")),
            super::Token::RAr,
            super::Token::RSq,
            super::Token::LSq,
            super::Token::FlagShort,
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

    #[test]
    fn cmd3() {
        let input = "[-b] [-t <target-pane>] <shell-command>";
        let exp = vec![
            super::Token::LSq,
            super::Token::FlagShort,
            super::Token::Str(String::from("b")),
            super::Token::RSq,
            super::Token::LSq,
            super::Token::FlagShort,
            super::Token::Str(String::from("t")),
            super::Token::LAr,
            super::Token::Str(String::from("target-pane")),
            super::Token::RAr,
            super::Token::RSq,
            super::Token::LAr,
            super::Token::Str(String::from("shell-command")),
            super::Token::RAr,
            super::Token::Eof,
        ];
        let result = super::lex(&input);
        assert_eq!(result, exp);
    }

    #[test]
    fn cmd4() {
        let input = "[-s <session-name>] [-n <window-name>] [command]";
        let exp = vec![
            super::Token::LSq,
            super::Token::FlagShort,
            super::Token::Str(String::from("s")),
            super::Token::LAr,
            super::Token::Str(String::from("session-name")),
            super::Token::RAr,
            super::Token::RSq,
            super::Token::LSq,
            super::Token::FlagShort,
            super::Token::Str(String::from("n")),
            super::Token::LAr,
            super::Token::Str(String::from("window-name")),
            super::Token::RAr,
            super::Token::RSq,
            super::Token::LSq,
            super::Token::Str(String::from("command")),
            super::Token::RSq,
            super::Token::Eof,
        ];
        let result = super::lex(&input);
        assert_eq!(result, exp);
    }

    #[test]
    fn cmd5() {
        let input = "[-b] [-t <target-pane>] <shell-command> <command> [else-command]";
        let exp = vec![
            super::Token::LSq,
            super::Token::FlagShort,
            super::Token::Str(String::from("b")),
            super::Token::RSq,
            super::Token::LSq,
            super::Token::FlagShort,
            super::Token::Str(String::from("t")),
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
            super::Token::Eof,
        ];
        let result = super::lex(&input);
        assert_eq!(result, exp);
    }

    #[test]
    fn flag_with_or() {
        let input = "[-D | -U] [-t <target-pane>]";
        let exp = vec![
            super::Token::LSq,
            super::Token::FlagShort,
            super::Token::Str(String::from("D")),
            super::Token::Or,
            super::Token::FlagShort,
            super::Token::Str(String::from("U")),
            super::Token::RSq,
            super::Token::LSq,
            super::Token::FlagShort,
            super::Token::Str(String::from("t")),
            super::Token::LAr,
            super::Token::Str(String::from("target-pane")),
            super::Token::RAr,
            super::Token::RSq,
            super::Token::Eof,
        ];
        let result = super::lex(&input);
        assert_eq!(result, exp);
    }

    #[test]
    fn long_argument() {
        let input = "[-b <branch>] [--depth <depth>] <repository> [<directory>]";
        let exp = vec![
            super::Token::LSq,
            super::Token::FlagShort,
            super::Token::Str(String::from("b")),
            super::Token::LAr,
            super::Token::Str(String::from("branch")),
            super::Token::RAr,
            super::Token::RSq,
            super::Token::LSq,
            super::Token::FlagLong,
            super::Token::Str(String::from("depth")),
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
            super::Token::Eof,
        ];
        let result = super::lex(&input);
        assert_eq!(result, exp);
    }

    #[test]
    fn two_long_flags() {
        let input = "[--hard | --soft] <commit>";
        let exp = vec![
            super::Token::LSq,
            super::Token::FlagLong,
            super::Token::Str(String::from("hard")),
            super::Token::Or,
            super::Token::FlagLong,
            super::Token::Str(String::from("soft")),
            super::Token::RSq,
            super::Token::LAr,
            super::Token::Str(String::from("commit")),
            super::Token::RAr,
            super::Token::Eof
        ];
        let result = super::lex(&input);
        assert_eq!(result, exp);
    }
    
   #[test]
   fn command_and_subcommand() {
       let input = "git reset [--hard | --soft] <commit>";
       let exp = vec![
           super::Token::Str(String::from("git")),
           super::Token::Str(String::from("reset")),
           super::Token::LSq,
           super::Token::FlagLong,
           super::Token::Str(String::from("hard")),
           super::Token::Or,
           super::Token::FlagLong,
           super::Token::Str(String::from("soft")),
           super::Token::RSq,
           super::Token::LAr,
           super::Token::Str(String::from("commit")),
           super::Token::RAr,
           super::Token::Eof
       ];
       let result = super::lex(&input);
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
           super::Token::Eof
       ];
       let result = super::lex(&input);
       assert_eq!(result, exp);
   }
}
