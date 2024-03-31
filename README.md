# Rust Shell: A Custom Linux Shell Implementation

This project is a custom Linux shell created using Rust, featuring support for fundamental Linux commands.

## Supported Commands:

1. `echo`: Display input text
2. `pwd`: Print the current working directory
3. `ls`: List directory contents
4. `cat`: Display the contents of a file
5. `touch`: Create an empty file
6. `mkdir`: Create a new directory
7. `cd`: Change the current working directory
8. `grep`: Search for patterns in files
9.  `exit`: Exits the shell

## Options implemented
### `ls` command
- `-a`: Show hidden files and directories
- `-r`: Reverse the order of listing
- `-l`: Display detailed information in long format

### `grep` command
- `-i`: Performs case-insensitive pattern matching
- `-m <count>`: Limit the number of matched lines per file

### `echo` command
- `content >> filename`: Redirects content to the file specified

## Command syntax
### \`echo\`
    echo [text] >> filename
### \`pwd\`
    pwd
### \`ls\`
    ls [options]
### \`cat\`
    cat filename
### \`touch\`
    touch filename
### \`mkdir\`
    mkdir directory_name
### \`cd\`
    cd directory/folder/path
### \`grep\`
    grep search_pattern file_path [options] 
### \`exit\`
    exit

## Future Enhancements
- Support for piping
- Implementation of more advanced commands
  


**Note:** _At the moment, the `grep` command only recognizes a search term._
