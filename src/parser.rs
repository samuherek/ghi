use crate::lexer::{Token, CmdLexer};
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
enum BinaryOp {
    Or
}

#[derive(Debug, PartialEq, Clone)]
enum Variable {
    String,
    Int
}

/// This is the rough docs for the structure of the chunks
///
/// The input value
/// used for literal values like commands, subcommands
/// == Literal
/// - value: String
/// - required : Bool
///
/// The input value
/// used as a variable value like strings, paths, ...
/// == Variable
/// - name : String
/// - type: String | Int
/// - required : Bool
/// TODO: - multiple: Bool
///
/// == FlagShort
/// - value: Char
/// - input: Option<Input>
/// - required : Bool
///
/// == FlagLong
/// - value: String
/// - input: Option<Input>
/// - required : Bool
///
/// == FlagCombo
/// - values: Char[],
/// - required : Bool
///
/// == Or
/// - lhs: Inp,
/// - rhs: Inp,
#[derive(Debug, PartialEq, Clone)]
pub enum CmdWord {
    Literal {
        value: String,
        required: bool
    },
    Variable {
        name: String,
        kind: Variable,
        required: bool
    },
    FlagShort {
        value: char,
        input: Box<Option<CmdWord>>,
        required: bool
    },
    FlagLong {
        value: String,
        input: Box<Option<CmdWord>>,
        required: bool
    },
    FlagCombo {
        values: Vec<char>,
        required: bool
    },
    BinaryOp {
        op:  BinaryOp,
        lhs: Box<CmdWord>,
        rhs: Box<CmdWord>,
    }
}

// impl fmt::Display for CmdWord {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             CmdWord::Command(val) => write!(f, "{val}"),
//             CmdWord::Arg(val) => write!(f, "{val}"),
//             CmdWord::Chunk{content, required} => {
//                 let content = content.to_string(); 
//                 if *required {
//                     write!(f, "<{content}>")
//                 } else {
//                     write!(f, "[{content}]")
//                 }
//             }
//         }
//     }
// }

#[derive(Debug, PartialEq)]
pub struct CmdParser {
    lexer: Vec<Token>,
    curr_position: usize,
    curr_token: Option<Token>,
    peak_token: Option<Token>,
    errors: Vec<String>,
}

impl CmdParser {
    pub fn compile(input: &str) -> Vec<CmdWord> {
        let lexer = CmdLexer::compile(input);
        let mut parser =  Self {
            lexer,
            curr_position: 0,
            curr_token: None,
            peak_token: None,
            errors: Vec::new()
        };

        parser.next_token();
        parser.next_token();

        let mut ast = Vec::new();
        while parser.curr_token.is_some() {
            for item in parser.parse_exp(true) {
                 // println!("next round {:?}", item);
                if let Some(exp) = item {
                     // println!("exp:: {:?}",exp);
                    ast.push(exp);
                }
            }
            parser.next_token();
        };

        println!("AST::: {:?}", ast);

       return ast;
    }

    fn next_token(&mut self) {
        self.curr_token = self.peak_token.take(); 
        // TODO: this is just not idea way to do it.
        self.peak_token = self.lexer.get(self.curr_position).map(|x| x.clone());
        self.curr_position = self.curr_position + 1;
    }

    fn read_literal(&self) -> CmdWord {
        if let Some(Token::Str(value)) = &self.curr_token {
            CmdWord::Literal {
                value: value.to_string(),
                required: true,
            }
        } else {
            panic!("Called parse literal without string");
        }
    }

    fn read_variable(&self, required: bool) -> CmdWord {
        if let Some(Token::Str(value)) = &self.curr_token {
            CmdWord::Variable {
                name: value.to_string(),
                kind: Variable::String,
                required,
            }
        } else {
            panic!("Called parse variable without string");
        }
    }

    fn parse_exp(&mut self, required: bool) -> Vec<Option<CmdWord>> {
        let token = self.curr_token.clone().expect("Cluld not find token in the next token");
        let mut words = Vec::new();

        match token {
            // It is always literal in this case
            Token::Str(_) => {
                words.push(Some(self.read_literal()));
            },
            // if next token is `=` we have a required input (depth + 1)
            // if next token is `LSq` we have an input optional (depth + 1)
            // if next token is `LAr` we have an input required (depth + 1)
            Token::FlagShort(val) => { 
                let mut input: Option<CmdWord> = None;

                if self.peak_token == Some(Token::LAr) {
                    self.next_token();
                    input = self.parse_exp(true).iter().flatten().next().cloned();
                }

                words.push(Some(CmdWord::FlagShort {
                    value: val.chars().next().expect("Short flag has to have a flag name char"),
                    input: Box::new(input),
                    required 
                }));
            },
            // if next token is `=` we have a required input (depth + 1)
            // if next token is `LSq` we have an input optional (depth + 1)
            // if next token is `LAr` we have an input required (depth + 1)
            Token::FlagLong(val) => {
                let mut input: Option<CmdWord> = None;

                if self.peak_token == Some(Token::LAr) {
                    self.next_token();
                    input = self.parse_exp(true).iter().flatten().next().cloned();
                }

                words.push(Some(CmdWord::FlagLong {
                    value: val.clone(),
                    input: Box::new(input),
                    required
                }));
            },
            // we take the val and split it into smaller tags
            // it can not have any input
            Token::FlagCombo(val) => {
                if val.len() < 2 {
                    panic!("Flag combo does not have enough flags");
                }
                
                words.push(Some(CmdWord::FlagCombo {
                    values: val.chars().collect(),
                    required
                }));
            },
            // call self.parse_exp until the next token is RSq
            Token::LSq => {
                while self.peak_token != Some(Token::RSq) {
                    self.next_token();
                    for item in self.parse_exp(false) {
                        words.push(item);
                    }
                }
                self.next_token();

                if self.curr_token != Some(Token::RSq) {
                    panic!("LSq did not find closing tag and run out of tokens");
                }
            },
            // call self.parse_exp until the next token is RAr
            Token::LAr => {
                self.next_token();
                words.push(Some(self.read_variable(true)));
                self.next_token();

                if self.curr_token != Some(Token::RAr) {
                    panic!("LAr can onyl take one argument and needs closing tag '>' ");
                }
            },
            Token::Or => {
                // take the previous exp and combine it with the next exp
                todo!();
            },
            Token::Multiple => {
                // Take the previous exp and turn it into a vector
                todo!();
            },
            t => {
                panic!("Unimplemented token paresr for {:?}", t);
            }
        };

        words
    }

    fn parse_input(value: &str, required: bool, variable: bool) -> CmdWord {
        let value = value.to_string();
        if variable {
            CmdWord::Variable {
                name: value,
                kind: Variable::String,
                required: true
            }
        } else {
            CmdWord::Literal {
                value,
                required,
            }
        }
    }
}

fn parse_input(value: &str, required: bool, variable: bool) -> CmdWord {
    let value = value.to_string();
    if variable {
        CmdWord::Variable {
            name: value,
            kind: Variable::String,
            required: true
        }
    } else {
        CmdWord::Literal {
            value,
            required,
        }
    }
}

// tmux choose-client [-t <target-session>]
// tmux choose-client                       -> true
// tmux choose-client -t lkjsdflk           -> true
// tmux choose-client -t lkjsdflk lkjasd    -> false
// tmux choose-client -t                    -> false

// let input = "git add [<file>... | <directory>...]";
// let exp = vec![
//     super::Token::Str(String::from("git")),
//     super::Token::Str(String::from("add")),
//     super::Token::LSq,
//     super::Token::LAr,
//     super::Token::Str(String::from("file")),
//     super::Token::RAr,
//     super::Token::Multiple,
//     super::Token::Or,
//     super::Token::LAr,
//     super::Token::Str(String::from("directory")),
//     super::Token::RAr,
//     super::Token::Multiple,
//     super::Token::RSq,
//     super::Token::Eof
// ];
// let result = super::lex(&input);
//
// AST
// [
//  Command(git),
//  Command(add),
//
// ]

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_literal() {
        let parser = CmdParser::compile("git");

        assert_eq!(parser, vec![
                   CmdWord::Literal {
                       value: "git".to_string(),
                       required: true
                   }
        ]);
    }

    #[test]
    fn two_literal() {
        let parser = CmdParser::compile("git add");

        assert_eq!(parser, vec![
                   CmdWord::Literal {
                       value: "git".to_string(),
                       required: true
                   },
                   CmdWord::Literal {
                       value: "add".to_string(),
                       required: true
                   }
        ]);
    }

    #[test]
    fn command_with_required_input() {
        let parser = CmdParser::compile("git add <path>");

        assert_eq!(parser, vec![
                   CmdWord::Literal {
                       value: "git".to_string(),
                       required: true
                   },
                   CmdWord::Literal {
                       value: "add".to_string(),
                       required: true
                   },
                   CmdWord::Variable {
                       name: "path".to_string(),
                       kind: Variable::String,
                       required: true
                   }
        ]);
    }

    #[test]
    fn command_with_short_flag() {
        let parser = CmdParser::compile("some command -f");

        assert_eq!(parser, vec![
                   CmdWord::Literal {
                       value: "some".to_string(),
                       required: true
                   },
                   CmdWord::Literal {
                       value: "command".to_string(),
                       required: true
                   },
                   CmdWord::FlagShort {
                       value: 'f',
                       input: Box::new(None),
                       required: true
                   }
        ]);
    }

    #[test]
    fn cmd_with_short_flag_with_input() {
        let parser = CmdParser::compile("some command -f <value>");

        assert_eq!(parser, vec![
                   CmdWord::Literal {
                       value: "some".to_string(),
                       required: true
                   },
                   CmdWord::Literal {
                       value: "command".to_string(),
                       required: true
                   },
                   CmdWord::FlagShort {
                       value: 'f',
                       input: Box::new(Some(CmdWord::Variable {
                            name: "value".to_string(),
                            kind: Variable::String, 
                            required: true
                       })),
                       required: true
                   }
        ]);
    }

    #[test]
    fn command_with_long_flag() {
        let parser = CmdParser::compile("some command --depth");

        assert_eq!(parser, vec![
                   CmdWord::Literal {
                       value: "some".to_string(),
                       required: true
                   },
                   CmdWord::Literal {
                       value: "command".to_string(),
                       required: true
                   },
                   CmdWord::FlagLong {
                       value: "depth".to_string(),
                       input: Box::new(None),
                       required: true
                   }
        ]);
    }

    #[test]
    fn command_with_combo_flag() {
        let parser = CmdParser::compile("some command -la");

        assert_eq!(parser, vec![
                   CmdWord::Literal {
                       value: "some".to_string(),
                       required: true
                   },
                   CmdWord::Literal {
                       value: "command".to_string(),
                       required: true
                   },
                   CmdWord::FlagCombo {
                       values: vec!['l', 'a'],
                       required: true
                   }
        ]);
    }

    #[test]
    fn command_with_optional_short_flag() {
        let parser = CmdParser::compile("some command [-l]");

        assert_eq!(parser, vec![
                   CmdWord::Literal {
                       value: "some".to_string(),
                       required: true
                   },
                   CmdWord::Literal {
                       value: "command".to_string(),
                       required: true
                   },
                   CmdWord::FlagShort {
                       value: 'l',
                       input: Box::new(None),
                       required: false
                   }
        ]);
    }

    #[test]
    fn cmd_and_optional_short_flag_with_input() {
        let parser = CmdParser::compile("some command [-l <value>]");

        assert_eq!(parser, vec![
                   CmdWord::Literal {
                       value: "some".to_string(),
                       required: true
                   },
                   CmdWord::Literal {
                       value: "command".to_string(),
                       required: true
                   },
                   CmdWord::FlagShort {
                       value: 'l',
                       input: Box::new(Some(
                            CmdWord::Variable {
                                name: "value".to_string(),
                                kind: Variable::String,
                                required: true
                            }
                       )),
                       required: false
                   }
        ]);
    }
}
