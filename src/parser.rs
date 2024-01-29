use crate::lexer::{Token, CmdLexer};
use std::fmt;

#[derive(Debug, PartialEq)]
enum BinaryOp {
    Or
}

#[derive(Debug, PartialEq)]
enum VariableType {
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
#[derive(Debug, PartialEq)]
pub enum CmdChunk {
    Literal {
        value: String,
        required: bool
    },
    Variable {
        name: String,
        kind: VariableType,
        required: bool
    },
    FlagShort {
        value: char,
        input: Box<Option<CmdChunk>>,
        required: bool
    },
    FlagLong {
        value: String,
        input: Box<Option<CmdChunk>>,
        required: bool
    },
    FlagCombo {
        values: Vec<char>,
        required: bool
    },
    BinaryOp {
        op:  BinaryOp,
        lhs: Box<CmdChunk>,
        rhs: Box<CmdChunk>,
    }
}

// impl fmt::Display for CmdChunk {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             CmdChunk::Command(val) => write!(f, "{val}"),
//             CmdChunk::Arg(val) => write!(f, "{val}"),
//             CmdChunk::Chunk{content, required} => {
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
    pub fn compile(input: &str) -> Vec<CmdChunk> {
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
           let exp = parser.parse_exp(true);
           ast.push(exp);
           parser.next_token();
        };

       return ast;
    }

    fn next_token(&mut self) {
        self.curr_token = self.peak_token.take(); 
        // TODO: this is just not idea way to do it.
        self.peak_token = self.lexer.get(self.curr_position).map(|x| x.clone());
        self.curr_position = self.curr_position + 1;
    }

    fn parse_exp(&mut self, required: bool) -> CmdChunk {
        let token = self.curr_token.clone().expect("Cluld not find token in the next token");

        let token = match token {
            // It is always literal in this case
            Token::Str(val) => {
                parse_input(&val, required, false)
            },
            // if next token is `=` we have a required input (depth + 1)
            // if next token is `LSq` we have an input optional (depth + 1)
            // if next token is `LAr` we have an input required (depth + 1)
            Token::FlagShort(val) => { 
                let mut input: Option<CmdChunk> = None;

                match self.peak_token.as_ref() {
                    Some(Token::LAr) => {
                        input = Some(self.parse_exp(required));
                    },
                    _ => {}
                };

                CmdChunk::FlagShort {
                    value: val.chars().next().expect("Short flag has to have a flag name char"),
                    input: Box::new(input),
                    required 
                }
            },
            // if next token is `=` we have a required input (depth + 1)
            // if next token is `LSq` we have an input optional (depth + 1)
            // if next token is `LAr` we have an input required (depth + 1)
            Token::FlagLong(val) => {
                let mut input: Option<CmdChunk> = None;

                match self.peak_token.as_ref() {
                    Some(Token::LAr) => {
                        input = Some(self.parse_exp(true));
                    },
                    _ => {}
                };

                
                CmdChunk::FlagLong {
                    value: val.clone(),
                    input: Box::new(input),
                    required
                }
            },
            // we take the val and split it into smaller tags
            // it can not have any input
            Token::FlagCombo(val) => {
                if val.len() < 2 {
                    panic!("Flag combo does not have enough flags");
                }
                
                CmdChunk::FlagCombo {
                    values: val.chars().collect(),
                    required
                }
            },
            // call self.parse_exp until the next token is RSq
            Token::LSq => {
                self.parse_exp(false)
            },
            // call self.parse_exp until the next token is RAr
            Token::LAr => {
                self.next_token();

                if let Some(Token::Str(val)) = &self.curr_token {
                    let output = parse_input(&val, true, true);
                    self.next_token(); 
                    output
                } else {
                    panic!("Next token is not a string but {:?}", self.curr_token);
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

        token

        // match token {
        //     Token::Str(val) => CmdChunk::Literal { 
        //         value: val.clone(), 
        //         required: true 
        //     },
        //     Token::FlagShort(val) => CmdChunk::FlagShort { 
        //         val: val.chars().next().expect("Has to have some flag identifier"),
        //         input: self.parse_chunk(),
        //         values: val.chars().map(|x| x.to_string()).collect() 
        //     },
        //     Token::FlagLong(val) => CmdChunk::Flag{ 
        //         values: vec![val.clone()] 
        //     },
        //     Token::LAr => {
        //         let mut content = Vec::new();
        //         self.next_token();
        //         loop {
        //             content.push(self.parse_chunk());
        //             self.next_token();
        //             if let Some(Token::RAr) = self.curr_token {
        //                 break;
        //             }
        //         }
        //
        //         CmdChunk::Chunk {
        //             content,
        //             required: true
        //         }
        //     },
        //     Token::LSq => {
        //         let mut content = Vec::new();
        //         self.next_token();
        //
        //         loop {
        //             content.push(self.parse_chunk());
        //             self.next_token();
        //             if let Some(Token::RSq) = self.curr_token {
        //                 break;
        //             }
        //         }
        //
        //         CmdChunk::Chunk {
        //             content,
        //             required: false
        //         }
        //     },
        //     // TODO: It reaches this at some point when the string is empty
        //     v => {
        //         panic!("Got unknow token {:?}", v);
        //     }
        // }
    }

    fn parse_input(value: &str, required: bool, variable: bool) -> CmdChunk {
        let value = value.to_string();
        if variable {
            CmdChunk::Variable {
                name: value,
                kind: VariableType::String,
                required: true
            }
        } else {
            CmdChunk::Literal {
                value,
                required,
            }
        }
    }
}

fn parse_input(value: &str, required: bool, variable: bool) -> CmdChunk {
    let value = value.to_string();
    if variable {
        CmdChunk::Variable {
            name: value,
            kind: VariableType::String,
            required: true
        }
    } else {
        CmdChunk::Literal {
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
                   CmdChunk::Literal {
                       value: "git".to_string(),
                       required: true
                   }
        ]);
    }

    #[test]
    fn two_literal() {
        let parser = CmdParser::compile("git add");

        assert_eq!(parser, vec![
                   CmdChunk::Literal {
                       value: "git".to_string(),
                       required: true
                   },
                   CmdChunk::Literal {
                       value: "add".to_string(),
                       required: true
                   }
        ]);
    }

    #[test]
    fn command_with_required_input() {
        let parser = CmdParser::compile("git add <path>");

        assert_eq!(parser, vec![
                   CmdChunk::Literal {
                       value: "git".to_string(),
                       required: true
                   },
                   CmdChunk::Literal {
                       value: "add".to_string(),
                       required: true
                   },
                   CmdChunk::Variable {
                       name: "path".to_string(),
                       kind: VariableType::String,
                       required: true
                   }
        ]);
    }



    // #[test]
    // fn parse_optional_block() {
    //     let parser = CmdParser::compile("[file]");
    //
    //     assert_eq!(parser, vec![
    //                CmdChunk::Chunk{
    //                    content: vec![CmdChunk::Arg(String::from("file"))],
    //                    required: false
    //                },
    //     ]);
    // }
    //
    // #[test]
    // fn parse_optional_blocks() {
    //     let parser = CmdParser::compile("[file] [directory]");
    //
    //     assert_eq!(parser, vec![
    //         CmdChunk::Chunk {
    //             content: vec![CmdChunk::Arg(String::from("file"))],
    //             required: false,
    //         },
    //         CmdChunk::Chunk{
    //             content: vec![CmdChunk::Arg(String::from("directory"))],
    //             required: false,
    //         },
    //     ]);
    // }
    //
    // #[test]
    // fn parse_required_blocks() {
    //     let parser = CmdParser::compile("<file> <directory>");
    //
    //     assert_eq!(parser, vec![
    //         CmdChunk::Chunk{
    //             content: vec![CmdChunk::Arg(String::from("file"))],
    //             required: true
    //         },
    //         CmdChunk::Chunk{
    //             content: vec![CmdChunk::Arg(String::from("directory"))],
    //             required: true
    //         },
    //     ]);
    // }
    //
    // #[test]
    // fn parse_short_flags() {
    //     let parser = CmdParser::compile("-f -r -rf");
    //
    //     assert_eq!(parser, vec![
    //         CmdChunk::Flag{
    //             values: vec![String::from("f")]
    //         },
    //         CmdChunk::Flag{
    //             values: vec![String::from("r")]
    //         },
    //         CmdChunk::Flag{
    //             values: vec![String::from("r"), String::from("f")]
    //         },
    //     ]);
    // }
    //
    // #[test]
    // fn parse_long_flags() {
    //     let parser = CmdParser::compile("--hey --depth");
    //
    //     assert_eq!(parser, vec![
    //         CmdChunk::Flag{
    //             values: vec![String::from("hey")]
    //         },
    //         CmdChunk::Flag{
    //             values: vec![String::from("depth")]
    //         },
    //     ]);
    // }
}
