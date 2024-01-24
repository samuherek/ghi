use super::parser::{CmdPart, CmdParser};
use super::lexer;
use super::input_lexer;
use super::input_lexer::Token;


#[derive(Debug, PartialEq)]
struct CmdCompare {
    schema: Vec<CmdPart>,
    input: Vec<Token>,
}

impl CmdCompare {
    fn new(cmd: &str, input: &str) -> Self {
        Self {
            schema: CmdParser::new(lexer::lex(cmd)).parse_cmd(),
            input: input_lexer::lex(input),
        }
    }

    /// parser:: CmdPart::Argument(String::from("git"))
    /// input:: Token::Input(String::from("paste-buffer")),
    fn run(&self) -> bool {
        for (i, part) in self.schema.iter().enumerate() {
            let input_part = self.input.get(i);
            if input_part.is_none() {
                return false;
            }

            match part {
                CmdPart::Argument(val) => {
                    if let Some(Token::Input(input_part)) = input_part {
                        if val != input_part {
                            return false
                        }
                    }
                }, 
                _ => {}
            }
            println!("{}, {:?}", i, part);
        }

        return true
    }
}


mod tests {
    use super::*;

    #[test]
    fn init_empty_compare() {
        let comp = CmdCompare::new(&"", &"");

        assert_eq!(comp, CmdCompare {
            schema: Vec::new(),
            input: Vec::new()
        });
    }

    #[test]
    fn compare_eq_string() {
        let comp = CmdCompare::new(&"git", &"git").run();

        assert_eq!(comp, true);
    }

    #[test]
    fn compare_noteq_string() {
        let comp = CmdCompare::new(&"git", &"gits").run();

        assert_eq!(comp, false);
    }

    #[test]
    fn compare_two_eq_sting() {
        let comp = CmdCompare::new(&"git add", &"git add").run();

        assert_eq!(comp, true);
    }

    #[test]
    fn compare_two_noteq_sting() {
        let comp = CmdCompare::new(&"git add", &"git commit").run();

        assert_eq!(comp, false);
    }
}
