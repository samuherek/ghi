
![Ghi](https://github.com/samuherek/ghi/assets/5614385/784fc006-d875-4300-b113-f0644db4da69)


### How to use

Add a new item to the history of things to save
```sh
$ ghi add "command" 
```

List all the items in the saved history
```sh
$ ghi list
```

Test your knowledge on some some predetermined items
```sh
$ ghi test
```

Access interactive adding and removing from `bash_history` or `zsh_history`
```sh
$ ghi
```

----

Pattern matching to read:
- https://spacy.io/usage/rule-based-matching/
- https://www.geeksforgeeks.org/introduction-to-pattern-searching-data-structure-and-algorithm-tutorial/?ref=lbp
- https://www.geeksforgeeks.org/convert-infix-expression-to-postfix-expression/

Regex engine 
- https://deniskyashif.com/2020/08/17/parsing-regex-with-recursive-descent/
- https://deniskyashif.com/2019/02/17/implementing-a-regular-expression-engine/
- 


### TODO:

- [ ] Create a "bucket" where we can slip any "future" commands to learn and configure to learn. Like a reading list. It's great if you don't knwo what you want to do but want to add some commands to look into the next time. this would require two steps: 1. put things into a bucket 2. review/edit and move the item into a lesson

- [ ] responsive resizing of the terminal and the text wrapping/cutting/...


- [ ] interactive add of a command 
     - it will trigger an editor to add description
     - it will trigger an editor to provide translation of the command into a command language "ghi" uses internally
Example:
`git push --set-upstream origin feature/whatever-awesome-stuff`
should be converted to this
`git push --set-upstrem origin <branch>` 
This is to remove any kind of sensitive information, and it makes it much easier for ghi to parse it correctly and provide test feedback like duolingo


- [ ] basic flashcard -> I'm working on this now
    - Creating a comperator function that walks the ASTs and compares them. 
    - I should use the same idea how regular expressions work behind the scene. A state machine with backtracing to walk through the optional parts. 

- [ ] create a configuration object that sets the storage path (I want to add it to synology to share across computers)


### DONE
- [-] render the question/success/failure/ in a box in the middle of the screen

- [-] Setup a cursor tracking and syncing with the new diff

- [-] remove items from the existing list.

- [-] add command to add custom command from input

- [-] remove duplicates from the history

- [-] hitting enter in the search list will add it in and out of the list.

- [-] add a list command to list all available commands and search through them

- [-] highlight the already selected commands in the search list.

- [-] removing query to emtpy one does not reset the serach after the 1 char left.

- [-] enter submits the comand and saves it into .config files

- [-] add the build binary to my paths so I can start using it on the laptop


### Not applicable

[ ] ghi tail command for taking the last run command and save it (for users of tmux and smilar, the `zsh_history` or `bash_history` does not work. Maybe we need to use the `fc -ln -1` command to fetch the last command)
- this is no longer relevant. It turns out there is too many behaviors and it's hard to do this right. Now it only supports the "ghi add" that adds as text or you can pipe. 


--- 

### Why?
I wanted to get more familiar with command line tools, but I don't use new commands enough to learn them and remember then. Thus, this tool has a few things that solves this prolem:
- You can find and select the command from history
- It will store this reference in an `.md` file so it can be changed within a text editor (TODO)
- When you have a free time, you can flashcard these commands with command name and then description shown afterwards (TODO)

This way, I can stay in the command line, learn new less common commands and avoid using google. 
Also, I can commit this to git repo for future reference and easy laoding on a different computer.

Additional features:
- Add a command manually without the history search (TODO)
- ...
  
