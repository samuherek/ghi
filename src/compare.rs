use super::parser::{CmdChunk, CmdParser};
use super::lexer::{CmdLexer};
use super::input_lexer;
use super::input_lexer::Token;


fn match_schema(ast: &Vec<CmdChunk>, tokens: &Vec<Token>, ast_idx: usize, token_idx: usize) -> Vec<(String, bool)> {
    let mut curr_ast_idx = ast_idx;
    let mut curr_token_idx = token_idx;
    let mut res: Vec<(String, bool)> = Vec::new();

    while curr_ast_idx < ast.len() && curr_token_idx < tokens.len() {
        match (&ast[curr_ast_idx], &tokens[curr_token_idx]) {
            (CmdChunk::Command(cmd), Token::Input(word)) if cmd == word => {
                res.push((cmd.clone(), true));
                curr_ast_idx += 1;
                curr_token_idx += 1;
            },
            (CmdChunk::Arg(cmd), Token::Input(word)) if cmd == word => {
                res.push((cmd.clone(), true));
                curr_ast_idx += 1;
                curr_token_idx += 1;
            },
            (CmdChunk::Flag{ values }, Token::FlagShort(flag)) => {
                curr_ast_idx += 1;
                curr_token_idx += 1;
                let mut correct = false;
                let mut f = String::new();

                let flags: Vec<char> = flag.chars().collect();
                if values.len() == flags.len() && values.iter().all(|x| x.chars().next().is_some_and(|xx| flags.contains(&xx))) {
                    correct = true;
                    f.push('-');
                    f.push_str(flag.as_str());
                }

                res.push((f, correct));
            },
            (CmdChunk::Flag{ values }, Token::FlagLong(flag)) => {
                curr_ast_idx += 1;
                curr_token_idx += 1;
                let mut correct = false;
                let mut f = String::new();

                if values.len() == 1 && values.iter().next().is_some_and(|x| x == flag) {
                    correct = true;
                    f.push_str("--");
                    f.push_str(flag.as_str());
                }

                res.push((f, correct));
            },
            _ => {
                res.push(("Unknow".to_string(), false));
                break; 
            }
        }
    }

    return res 
}

mod tests {
    use super::*;


    #[test]
    fn match_matching_schema() {
        let tests = vec![
            "git",
            "-g",
            "--depth"
        ];

        for t in tests {
            let ast = CmdParser::compile(t);
            let input = input_lexer::lex(t);
            println!("{:?}", ast);
            println!("{:?}", input);
            let matcher = match_schema(&ast, &input, 0, 0);

            for (val, correctness) in matcher {
                assert_eq!(val, t);
                assert_eq!(correctness, val == t);
            }
        }
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
