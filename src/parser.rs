use crate::lexer::Token;

type Lexer = Vec<Token>;

#[derive(Debug, PartialEq)]
enum CmdPart {
    Input(String),
    Optional {
        blocks: Vec<CmdPart>
    },
    Required {
        blocks: Vec<CmdPart>
    }
}

#[derive(Debug, PartialEq)]
struct CmdParser {
    lexer: Lexer,
    curr_position: usize,
    curr_token: Option<Token>,
    peak_token: Option<Token>,
    errors: Vec<String>,
}

impl CmdParser {
    fn new(lexer: Lexer) -> Self {
        let mut parser =  Self {
            lexer,
            curr_position: 0,
            curr_token: None,
            peak_token: None,
            errors: Vec::new()
        };

        parser.next_token();
        parser.next_token();

       return parser;
    }

    fn next_token(&mut self) {
        self.curr_token = self.peak_token.take(); 
        // TODO: this is just not idea way to do it.
        self.peak_token = self.lexer.get(self.curr_position).map(|x| x.clone());
        self.curr_position = self.curr_position + 1;
    }

    fn parse_block(&mut self) -> CmdPart {
        match &self.curr_token {
            Some(Token::Str(val)) => CmdPart::Input(val.clone()),
            Some(Token::LAr) => {
                let mut blocks = Vec::new();
                self.next_token();
                loop {
                    blocks.push(self.parse_block());
                    self.next_token();
                    if let Some(Token::RAr) = self.curr_token {
                        break;
                    }
                }

                CmdPart::Required {
                    blocks 
                }
            },
            Some(Token::LSq) => {
                let mut blocks = Vec::new();
                self.next_token();

                loop {
                    blocks.push(self.parse_block());
                    self.next_token();
                    if let Some(Token::RSq) = self.curr_token {
                        break;
                    }
                }

                CmdPart::Optional {
                    blocks,                    
                }
            },
            _ => unimplemented!()
        }
    }

    fn parse_cmd(&mut self) -> Vec<CmdPart> {
        let mut res = Vec::new();
        while self.curr_token.is_some() {
           let part = self.parse_block();
           res.push(part);
           self.next_token();
        };

        res
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
        let parser = CmdParser::new(vec![
            Token::Str(String::from("git")),
        ]).parse_cmd();

        assert_eq!(parser, vec![
            CmdPart::Input(String::from("git"))
        ]);
   }

    #[test]
    fn parse_command_block() {
        let parser = CmdParser::new(vec![
            Token::Str(String::from("git")),
        ]).parse_cmd();

        assert_eq!(parser, vec![
            CmdPart::Input(String::from("git"))
        ]);
    }

    #[test]
    fn parse_command_and_subcomand_block() {
        let parser = CmdParser::new(vec![
            Token::Str(String::from("git")),
            Token::Str(String::from("add")),
        ]).parse_cmd();

        assert_eq!(parser, vec![
            CmdPart::Input(String::from("git")),
            CmdPart::Input(String::from("add"))
        ]);
    }

    #[test]
    fn parse_optional_block() {
        let parser = CmdParser::new(vec![
            Token::LSq,
            Token::Str(String::from("file")),
            Token::RSq,
        ]).parse_cmd();

        assert_eq!(parser, vec![
            CmdPart::Optional{
                blocks: vec![CmdPart::Input(String::from("file"))]
            },
        ]);
    }

    #[test]
    fn parse_optional_blocks() {
        let parser = CmdParser::new(vec![
            Token::LSq,
            Token::Str(String::from("file")),
            Token::RSq,
            Token::LSq,
            Token::Str(String::from("directory")),
            Token::RSq,
        ]).parse_cmd();

        assert_eq!(parser, vec![
            CmdPart::Optional{
                blocks: vec![CmdPart::Input(String::from("file"))]
            },
            CmdPart::Optional{
                blocks: vec![CmdPart::Input(String::from("directory"))]
            },
        ]);
    }

    #[test]
    fn parse_required_blocks() {
        let parser = CmdParser::new(vec![
            Token::LAr,
            Token::Str(String::from("file")),
            Token::RAr,
            Token::LAr,
            Token::Str(String::from("directory")),
            Token::RAr,
        ]).parse_cmd();

        assert_eq!(parser, vec![
            CmdPart::Required{
                blocks: vec![CmdPart::Input(String::from("file"))]
            },
            CmdPart::Required{
                blocks: vec![CmdPart::Input(String::from("directory"))]
            },
        ]);
    }
}
