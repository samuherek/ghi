use super::parser::{CmdWord, CmdParser};
use super::lexer::{CmdLexer};
use super::input_lexer::InputCmdLexer;
use super::input_lexer::Token;



//
// schema: git add .
// input: git add
//
// match: literal, match: literal
//
// compare: (failure)
// git add "src"
// ___ ___  xxx
// schema: git commit -m "init"
// input: git commit "init"
//
// match:literal, match:literal, missing, match:variable
//
// compare: (failure)
// git commit -m "init" 
// ___ ______ xxx _____
//
// schema: display-message -p -t client-0 "hello, world" 
// input: display-message -p -t client-0 "hello, world" 
//
// compare: (success)
// display-message -p -t client-0 "hello, world" 
// --------------- -- -- -------- --------------
//
// schema: display-message -p -t client-0 "hello, world" 
// input: display-message -t client-0 "hello, world" 
//
// compare: (failure)
// display-message -p -t client-0 "hello, world" 
// --------------- xx -- -------- --------------
//
// schema: display-message -p -t client-0 "hello, world" 
// input: display-message client-0 "hello, world" 
//
// compare: (failure)
// display-message -p -t client-0 "hello, world" 
// --------------- xx xx -------- --------------
//
// schema: display-message -p -t client-0 "hello, world" 
// input: display-message -p "hello, world" 
//
// compare: (failure)
// display-message -p -t client-0 "hello, world" 
// --------------- -- xx xxxxxxxx --------------
//
// schema: display-message -p -t client-0 "hello, world" 
// input: display-message "hello, world" 
//
// compare: (failure)
// display-message -p -t client-0 "hello, world" 
// --------------- xx xx xxxxxxxx --------------
//
// edge case which we probably don't want to support.
// schema: display-message -p -t client-0 "hello, world" 
// input: display-message -t -p client-0 "hello, world" 
//
// compare: (success)
// display-message -t -p client-0 "hello, world" 
// --------------- -- -- -------- --------------
/// 
/// [
///     Match(
///         ast: CmdWord,
///         input: Option<String>,
///         node: CmdWord
///     ),
///     Mismatch(
///         input: Option<String>,
///         node: CmdWord
///     )
/// ]
fn match_schema(ast: &Vec<CmdWord>, tokens: &Vec<Token>, ast_idx: usize, token_idx: usize) -> Vec<(String, bool)> {
    let mut curr_ast_idx = ast_idx;
    let mut curr_token_idx = token_idx;
    let mut res: Vec<(String, bool)> = Vec::new();

    while curr_ast_idx < ast.len() && curr_token_idx < tokens.len() {
        let cmd = &ast[curr_ast_idx];
        let token = &tokens[curr_token_idx];

        match (cmd, token) {
            (CmdWord::Literal{value}, Token::Str(word)) => {
                res.push((cmd.to_string(), value == word));
                curr_ast_idx += 1;
                curr_token_idx += 1;
            },
            (CmdWord::FlagShort{value, ..}, Token::FlagShort(word)) => {
                res.push((cmd.to_string(), value == word));
                curr_ast_idx += 1;
                curr_token_idx += 1;
            },
            (CmdWord::FlagLong{value, ..}, Token::FlagLong(word)) => {
                res.push((cmd.to_string(), value == word));
                curr_ast_idx += 1;
                curr_token_idx += 1;
            },
            (CmdWord::FlagCombo{values}, Token::FlagCombo(chars)) => {
                res.push((cmd.to_string(), values == chars));
                curr_ast_idx += 1;
                curr_token_idx += 1;
            }
            (cmd, token) => {
                unimplemented!("missing impl for cmd {} and token {}", cmd.to_string(), token.to_string());
            }
        }

        // match (&ast[curr_ast_idx], &tokens[curr_token_idx]) {
        //     (CmdWord::Command(cmd), Token::Str(word)) if cmd == word => {
        //         res.push((cmd.clone(), true));
        //         curr_ast_idx += 1;
        //         curr_token_idx += 1;
        //     },
        //     (CmdWord::Arg(cmd), Token::Str(word)) if cmd == word => {
        //         res.push((cmd.clone(), true));
        //         curr_ast_idx += 1;
        //         curr_token_idx += 1;
        //     },
        //     (CmdWord::Flag{ values }, Token::FlagShort(flag)) => {
        //         curr_ast_idx += 1;
        //         curr_token_idx += 1;
        //         let mut correct = false;
        //         let mut f = String::new();
        //         let flags: Vec<char> = flag.chars().collect();
        //         let all_flags_match = values.iter().all(|x| x.chars().next().is_some_and(|xx| flags.contains(&xx)));
        //
        //         if values.len() == flags.len() &&  all_flags_match {
        //             correct = true;
        //             f.push('-');
        //             f.push_str(flag.as_str());
        //         }
        //
        //         res.push((f, correct));
        //     },
        //     (CmdWord::Flag{ values }, Token::FlagLong(flag)) => {
        //         curr_ast_idx += 1;
        //         curr_token_idx += 1;
        //         let mut correct = false;
        //         let mut f = String::new();
        //         let flag_matches = values.iter().next().is_some_and(|x| x == flag);
        //
        //         if values.len() == 1 && flag_matches {
        //             correct = true;
        //             f.push_str("--");
        //             f.push_str(flag.as_str());
        //         }
        //
        //         res.push((f, correct));
        //     },
        //     // (CmdWord::Chunk{ content, required }, Token::Input(val)) => {
        //     //     if *required {
        //     //         curr_ast_idx += 1;
        //     //         curr_token_idx += 1;
        //     //         if content.len() == 1 {
        //     //             if let Some(CmdWord::Arg(tag)) = content.get(0) {
        //     //                 res.push((tag.clone(), true));
        //     //             }
        //     //         }
        //     //     }
        //     // },
        //     (_, token) => {
        //         res.push(("Unknow".to_string(), false));
        //         break; 
        //     }
        // }
    }

    return res 
}



mod tests {
    use super::*;


    #[test]
    fn match_single_item() {
        let tests = vec![
            ("git", "git"),
            ("-f", "-f"),
        ];

        for (cmd, s) in tests {
            let ast = CmdParser::compile(cmd);
            let input = InputCmdLexer::compile(s);
            let matcher = match_schema(&ast, &input, 0, 0);

            for (val, correctness) in matcher {
                assert_eq!(val, s);
                assert_eq!(correctness, val == s);
            }
        }
    }

    #[test]
    fn match_multiple_items() {
        let val = "git add";
        let ast = CmdParser::compile(&val);
        let input = InputCmdLexer::compile(&val);
        let matcher = match_schema(&ast, &input, 0, 0);
        let splits = val.split_whitespace().collect::<Vec<_>>();

        for (i, (cmd, s)) in matcher.iter().enumerate() {
            let split = splits[i];
           assert_eq!(cmd, split);
           assert_eq!(*s, cmd.as_str() == split);
        }
    }

    #[test]
    fn match_short_flag() {
        let val = "git -f";
        let ast = CmdParser::compile(&val);
        let input = InputCmdLexer::compile(&val);
        let matcher = match_schema(&ast, &input, 0, 0);
        let splits = val.split_whitespace().collect::<Vec<_>>();

        for (i, (cmd, s)) in matcher.iter().enumerate() {
            let split = splits[i];
           assert_eq!(cmd, split);
           assert_eq!(*s, cmd.as_str() == split);
        }
    }

    #[test]
    fn match_long_flag() {
        let val = "git --depht";
        let ast = CmdParser::compile(&val);
        let input = InputCmdLexer::compile(&val);
        let matcher = match_schema(&ast, &input, 0, 0);
        let splits = val.split_whitespace().collect::<Vec<_>>();

        for (i, (cmd, s)) in matcher.iter().enumerate() {
            let split = splits[i];
           assert_eq!(cmd, split);
           assert_eq!(*s, cmd.as_str() == split);
        }
    }

    #[test]
    fn match_combo_flag() {
        let val = "git -la";
        let ast = CmdParser::compile(&val);
        let input = InputCmdLexer::compile(&val);
        let matcher = match_schema(&ast, &input, 0, 0);
        let splits = val.split_whitespace().collect::<Vec<_>>();

        for (i, (cmd, s)) in matcher.iter().enumerate() {
            let split = splits[i];
           assert_eq!(cmd, split);
           assert_eq!(*s, cmd.as_str() == split);
        }
    }
    // #[test]
    // fn match_multi_item() {
    //     let ast = CmdParser::compile("git add");
    //     let input = InputCmdLexer::compile("git add");
    //     let matcher = match_schema(&ast, &input, 0, 0);
    //     
    //     assert_eq!(matcher, vec![
    //        ("git".to_string(), true),
    //        ("add".to_string(), true)
    //     ]);
    //
    //     assert_ne!(matcher, vec![
    //        ("git".to_string(), false),
    //        ("add".to_string(), true)
    //     ]);
    // }
    //
    // #[test]
    // fn match_multi_item2() {
    //     let ast = CmdParser::compile("git add");
    //     let input = InputCmdLexer::compile("git commit");
    //     let matcher = match_schema(&ast, &input, 0, 0);
    //     
    //     assert_eq!(matcher, vec![
    //        ("git".to_string(), true),
    //        ("add".to_string(), false)
    //     ]);
    // }


}
