use crate::lexer::{Token, CmdLexer};
use std::fmt;

#[derive(Debug, PartialEq)]
enum BinaryOp {
    Or
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
    
    Command(String),
    Arg(String),
    Chunk {
        content: Vec<CmdChunk>,
        required: bool
    },
    Flag {
        values: Vec<String>,
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
           let chunk = parser.parse_chunk();
           ast.push(chunk);
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

    fn parse_chunk(&mut self) -> CmdChunk {
        let token = self.curr_token.as_ref().expect("Cluld not find token in the next token");

        match token {
            Token::Str(val) => {
                if self.curr_position == 2 {
                    CmdChunk::Command(val.clone())
                } else {
                    CmdChunk::Arg(val.clone())
                }
            },
            Token::FlagShort(val) => CmdChunk::Flag{ values: val.chars().map(|x| x.to_string()).collect() },
            Token::FlagLong(val) => CmdChunk::Flag{ values: vec![val.clone()] },
            Token::LAr => {
                let mut content = Vec::new();
                self.next_token();
                loop {
                    content.push(self.parse_chunk());
                    self.next_token();
                    if let Some(Token::RAr) = self.curr_token {
                        break;
                    }
                }

                CmdChunk::Chunk {
                    content,
                    required: true
                }
            },
            Token::LSq => {
                let mut content = Vec::new();
                self.next_token();

                loop {
                    content.push(self.parse_chunk());
                    self.next_token();
                    if let Some(Token::RSq) = self.curr_token {
                        break;
                    }
                }

                CmdChunk::Chunk {
                    content,
                    required: false
                }
            },
            // TODO: It reaches this at some point when the string is empty
            v => {
                println!("{:?}", v);
                unimplemented!()
            }
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
    fn creates_parser() {
        let parser = CmdParser::compile("git");

        assert_eq!(parser, vec![
            CmdChunk::Arg(String::from("git"))
        ]);
   }

    #[test]
    fn just_once_letter() {
        let parser = CmdParser::compile("a");

        assert_eq!(parser, vec![
            CmdChunk::Arg(String::from("a"))
        ]);
    }

    #[test]
    fn parse_command_block() {
        let parser = CmdParser::compile("git");

        assert_eq!(parser, vec![
            CmdChunk::Arg(String::from("git"))
        ]);
    }

    #[test]
    fn parse_command_and_subcomand_block() {
        let parser = CmdParser::compile("git add");

        assert_eq!(parser, vec![
            CmdChunk::Arg(String::from("git")),
            CmdChunk::Arg(String::from("add"))
        ]);
    }

    #[test]
    fn parse_optional_block() {
        let parser = CmdParser::compile("[file]");

        assert_eq!(parser, vec![
                   CmdChunk::Chunk{
                       content: vec![CmdChunk::Arg(String::from("file"))],
                       required: false
                   },
        ]);
    }

    #[test]
    fn parse_optional_blocks() {
        let parser = CmdParser::compile("[file] [directory]");

        assert_eq!(parser, vec![
            CmdChunk::Chunk {
                content: vec![CmdChunk::Arg(String::from("file"))],
                required: false,
            },
            CmdChunk::Chunk{
                content: vec![CmdChunk::Arg(String::from("directory"))],
                required: false,
            },
        ]);
    }

    #[test]
    fn parse_required_blocks() {
        let parser = CmdParser::compile("<file> <directory>");

        assert_eq!(parser, vec![
            CmdChunk::Chunk{
                content: vec![CmdChunk::Arg(String::from("file"))],
                required: true
            },
            CmdChunk::Chunk{
                content: vec![CmdChunk::Arg(String::from("directory"))],
                required: true
            },
        ]);
    }

    #[test]
    fn parse_short_flags() {
        let parser = CmdParser::compile("-f -r -rf");

        assert_eq!(parser, vec![
            CmdChunk::Flag{
                values: vec![String::from("f")]
            },
            CmdChunk::Flag{
                values: vec![String::from("r")]
            },
            CmdChunk::Flag{
                values: vec![String::from("r"), String::from("f")]
            },
        ]);
    }

    #[test]
    fn parse_long_flags() {
        let parser = CmdParser::compile("--hey --depth");

        assert_eq!(parser, vec![
            CmdChunk::Flag{
                values: vec![String::from("hey")]
            },
            CmdChunk::Flag{
                values: vec![String::from("depth")]
            },
        ]);
    }
}
