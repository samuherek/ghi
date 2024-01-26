use super::parser::{CmdChunk, CmdParser};
use super::lexer::{CmdLexer};
use super::input_lexer;
use super::input_lexer::Token;


#[derive(Debug, PartialEq)]
struct CmdCompare {
    schema: Vec<CmdChunk>,
    input: Vec<Token>,
}

impl CmdCompare {
    fn new(cmd: &str, input: &str) -> Self {
        Self {
            schema: CmdParser::compile(cmd),
            input: input_lexer::lex(input),
        }
    }

    /// parser:: CmdChunk::Argument(String::from("git"))
    /// input:: Token::Input(String::from("paste-buffer")),
    fn run(&self) -> bool {
        for (i, part) in self.schema.iter().enumerate() {
            let input_part = self.input.get(i);
            if let Some(input_part) = input_part {
                if !compare_token(part, input_part) {
                    return false
                }
            } else {
                return false;
            }
            println!("{}, {:?}", i, part);
        }

        return true
    }
}

fn compare_token(cmd_token: &CmdChunk, input_token: &Token) -> bool {
    match cmd_token {
        CmdChunk::Arg(cmd_val) => {
            match input_token {
                Token::Input(input_val) =>  {
                    return cmd_val == input_val
                },
                _ => return false
            }
        },
        _ => {}
    }

    return false
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
    //
    // #[test]
    // fn compare_eq_flag() {
    //     let comp = CmdCompare::new(&"git -f", &"git -f").run();
    //
    //     assert_eq!(comp, true);
    // }
    //

    // schema: git add .
    // input: git add
    //
    // compare: (failure)
    // git add "src"
    // ___ ___  xxx
    
    // schema: git commit -m "init"
    // input: git commit "init"
    //
    // compare: (failure)
    // git commit -m "init" 
    // ___ ______ xxx _____
    
    // schema: display-message -p -t client-0 "hello, world" 
    // input: display-message -p -t client-0 "hello, world" 
    //
    // compare: (success)
    // display-message -p -t client-0 "hello, world" 
    // --------------- -- -- -------- --------------
    
    // schema: display-message -p -t client-0 "hello, world" 
    // input: display-message -t client-0 "hello, world" 
    //
    // compare: (failure)
    // display-message -p -t client-0 "hello, world" 
    // --------------- xx -- -------- --------------
    
    // schema: display-message -p -t client-0 "hello, world" 
    // input: display-message client-0 "hello, world" 
    //
    // compare: (failure)
    // display-message -p -t client-0 "hello, world" 
    // --------------- xx xx -------- --------------
    
    // schema: display-message -p -t client-0 "hello, world" 
    // input: display-message -p "hello, world" 
    //
    // compare: (failure)
    // display-message -p -t client-0 "hello, world" 
    // --------------- -- xx xxxxxxxx --------------
    
    // schema: display-message -p -t client-0 "hello, world" 
    // input: display-message "hello, world" 
    //
    // compare: (failure)
    // display-message -p -t client-0 "hello, world" 
    // --------------- xx xx xxxxxxxx --------------
    

    // edge case which we probably don't want to support.
    // schema: display-message -p -t client-0 "hello, world" 
    // input: display-message -t -p client-0 "hello, world" 
    //
    // compare: (success)
    // display-message -t -p client-0 "hello, world" 
    // --------------- -- -- -------- --------------
}
