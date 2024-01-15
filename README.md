
![Ghi](https://github.com/samuherek/ghi/assets/5614385/784fc006-d875-4300-b113-f0644db4da69)
The key ingredient to get you cooking in your code.

### How to use


Add a new item to the history of things to save
```sh
$ ghi add "command" 
```

List all the items in the saved history
```sh
$ ghi list
```

Access interactive adding and removing from `bash_history` or `zsh_history`
```sh
$ ghi
```


### TODO:
[ ] basic flashcard

[ ] add command to add custom command from input

[ ] ghi tail command for taking the last run command and save it (for users of tmux and smilar, the `zsh_history` or `bash_history` does not work. Maybe we need to use the `fc -ln -1` command to fetch the last command)

[ ] create a configuration object that sets the storage path (I want to add it to synology to share across computers)


### DONE
[-] remove items from the existing list.

[-] remove duplicates from the history

[-] hitting enter in the search list will add it in and out of the list.

[-] add a list command to list all available commands and search through them

[-] highlight the already selected commands in the search list.

[-] removing query to emtpy one does not reset the serach after the 1 char left.

[-] enter submits the comand and saves it into .config files

[-] add the build binary to my paths so I can start using it on the laptop

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
