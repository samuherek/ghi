use std::fs;
use std::path::PathBuf;
use std::io::stdin;
use crate::parser::CmdParser;
use crate::input_lexer::InputCmdLexer;
use crate::compare::match_schema;

pub fn run() {
    let data = fs::read_to_string(PathBuf::from("test.txt")).unwrap();
    let lines: Vec<_> = data.lines().filter(|x| !x.is_empty()).collect();
    let mut cmds = Vec::new();
    let mut line_iter = lines.iter();

    while let Some(line) = line_iter.next() {
        if line.starts_with('#') {
            let description = *line;
            if let Some(line) = line_iter.next() {
                cmds.push((description, *line));
            } else {
                panic!("didn't find command for a description in the field");
            }
        }
    }

    for cmd in cmds {
        println!("Lesson:");
        println!("{}", cmd.0);

        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(_) => {
                let ast = CmdParser::compile(cmd.1);
                let in_lex = InputCmdLexer::compile(&input.trim());
                let matcher = match_schema(&ast, &in_lex, 0, 0);
                let is_full_match = matcher.iter().all(|x| x.1);
                let msg = if is_full_match {
                    "correct"
                } else {
                    "wrong"
                };

                println!("You got it {msg}");
                let mut line = String::new();
                let mut underline = String::new();

                for (value, is_match) in matcher {
                    line.push_str(value.as_str());
                    if is_match {
                        underline.push_str(&" ".repeat(value.len()));
                    } else {
                        underline.push_str(&"^".repeat(value.len()));
                    }
                    line.push_str(" ");
                    underline.push_str(" ");
                }
                if !is_full_match {
                    println!("{line}");
                    println!("{underline}");
                }
                println!("");
            },
            Err(err) => {
                panic!("Error reading the input {}", err);
            }
        }
    }
}
