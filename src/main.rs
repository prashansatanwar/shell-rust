use std::io::{ self, stdin, stdout, BufRead, BufWriter, Write };
use std::env;
use std::fs::{self, File, OpenOptions};
use std::path::Path;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use colored::Colorize;
use chrono::{DateTime, Local};
use users::{get_user_by_uid, get_group_by_gid};

struct FileInfo {
    name: String,
    metadata: fs::Metadata
}

fn echo(args: Vec<&str>) {
    let msg = args.join(" ");
    if msg.contains(">>") {
        let parts = msg.splitn(2, ">>").collect::<Vec<&str>>();

        if parts.len() >= 2 {
            let content = parts[0];
            let path = parts[1].trim();
            
            if path.len() > 0 {
                let file = OpenOptions::new().write(true).create(true).append(true).open(path);

                match file {
                    Ok(mut file) => {

                        let mut writer = BufWriter::new(&mut file);
    
                        match writer.write_all(&content.as_bytes()) {
                            Ok(_) => {
                                match writer.flush() {
                                    Ok(_) => return,
                                    Err(e) => eprintln!("Error: {}",e)
                                }
                            },
                            Err(e) => eprintln!("Error: {}",e)
                        }
                    },
                    Err(e) => eprintln!("Error: {}",e)
                }
            }
        }

        return;
    }
    let msg = args.join(" ");
    println!("{}",msg);
}

fn ls(args: Vec<&str>) {
    let mut show_hidden = false;
    let mut reverse = false;
    let mut long_format = false;

    for arg in args {
        match arg {
            "-a" => show_hidden = true,
            "-r" => reverse = true,
            "-l" => long_format = true,
            _ => {}
        }
    }

    if let Ok(current_dir) = env::current_dir() {
        if let Ok(entries) = fs::read_dir(current_dir) {

            let mut max_length = 0;
            let mut files: Vec<FileInfo> = Vec::new();

            if show_hidden {
                files.push(FileInfo{name:".".to_string(), metadata: fs::metadata(".").unwrap() });
                files.push(FileInfo{name:"..".to_string(), metadata: fs::metadata("..").unwrap() });
            }

            for entry in entries {                
                if let Ok(entry) = entry {
                    if let Some(file_name) = entry.file_name().to_str() {

                        if !show_hidden && file_name.starts_with('.') {
                            continue;
                        }

                        if let Ok(metadata) = entry.metadata() {
                            files.push(FileInfo{
                                name:file_name.to_string(),
                                metadata
                            });
                        }

                        if file_name.len() > max_length {
                            max_length = file_name.len();
                        }
                    }
                }
            }

            files.sort_by(|a, b| a.name.cmp(&b.name));

            if reverse {
                files.reverse();
            }

            if long_format {
                for file in files {

                    let owner = match get_user_by_uid(file.metadata.uid()) {
                        Some(user) => user.name().to_string_lossy().into_owned(),
                        None => file.metadata.uid().to_string()
                    };

                    let group = match get_group_by_gid(file.metadata.gid()) {
                        Some(group) => group.name().to_string_lossy().into_owned(),
                        None => file.metadata.gid().to_string()
                    };

                    println!("{:>width$} {:>width$} {:>width$} {:>width$} {:>width$} {:>width$} {:>width$}",
                            mode_string(&file.metadata),
                            file.metadata.nlink(),
                            owner,
                            group,
                            file.metadata.size(),
                            DateTime::<Local>::from(file.metadata.modified().unwrap()).format("%b %e %H:%M").to_string(),                            
                            (if file.metadata.is_dir() {file.name.green()} else {file.name.normal()}),
                            width = 10)
                }
            }
            else {
                let column_width = max_length + 2;
                let num_columns = 100/column_width;
    
                for (i, file) in files.iter().enumerate() {
                    if i>0 && i%num_columns == 0 {
                        println!();
                    }
                    let entry_type = file.metadata.file_type();
                    if entry_type.is_dir() {
                        print!("{:<width$} ", file.name.green(), width = column_width);
                    }
                    else {
                        print!("{:<width$} ", file.name, width = column_width);
                    }
                }
                println!();
            }


        }
    }
}

fn mode_string(metadata: &fs::Metadata) -> String {
    let mut mode = String::with_capacity(9);

    // File type
    mode.push(if metadata.is_dir() {'d'} else {'-'});

    // Owner Permissions
    mode.push(if metadata.permissions().mode() & 0o400 != 0  {'r'} else {'-'});
    mode.push(if metadata.permissions().mode() & 0o200 != 0  {'w'} else {'-'});
    mode.push(if metadata.permissions().mode() & 0o100 != 0  {'x'} else {'-'});


    // Group Permissions
    mode.push(if metadata.permissions().mode() & 0o40 != 0  {'r'} else {'-'});
    mode.push(if metadata.permissions().mode() & 0o20 != 0  {'w'} else {'-'});
    mode.push(if metadata.permissions().mode() & 0o10 != 0  {'x'} else {'-'});


    // Others Permissions
    mode.push(if metadata.permissions().mode() & 0o4 != 0  {'r'} else {'-'});
    mode.push(if metadata.permissions().mode() & 0o2 != 0  {'w'} else {'-'});
    mode.push(if metadata.permissions().mode() & 0o1 != 0  {'x'} else {'-'});

    return mode;
}

fn cat(filename: String) {
    if let Ok(current_dir) = env::current_dir() {

        let filepath = current_dir.display().to_string() + "/"+ &filename;
        let file_content = fs::read_to_string(filepath);
        
        match file_content{
            Ok(file_content) => {
                println!("{}",file_content);
            }

            Err(e) => eprintln!("Error: {}",e) 
        };
    }
}

fn touch(filename: String) {
    let _ = fs::File::create(filename);
}

fn mkdir(dir: String) {
    let _ = fs::create_dir_all(dir);
}

fn cd(path: &str) {
    match env::set_current_dir(Path::new(path).to_path_buf()) {
        Ok(_) => {},
        Err(e) => eprintln!("Error: {}",e) 
    }
}

fn grep(search_pattern: String, paths: Vec<&str>, opts: Vec<&str>) {
    let mut max_count = i32::MAX;
    let mut case_sensitive = true;
    let mut count_limit = false;

    let mut opts_iter = opts.iter().peekable();
    while let Some(o) = opts_iter.next() {
            match *o {
                "-i" => case_sensitive = false,
                "-m" => {
                    count_limit = true;
                    if let Some(&next_arg) = opts_iter.peek() {
                        if let Ok(count) = next_arg.parse::<i32>() {
                            max_count = count;
                            opts_iter.next();
                        }
                        else {
                            eprintln!("Warning: '-m' option provided without a number.");
                        }
                    }
                },
                _ => {}
            }
    }

    for path in paths {
        let file = File::open(path);
        if let Ok(file) = file {
            let reader = io::BufReader::new(file);

            let mut line_number = 0;
            let mut count = 0;

            for line in reader.lines() {
                
                let mut print_line = false;
                
                if let Ok(line) = line {
                    line_number += 1;
                    
                    if case_sensitive {
                        if line.contains(&search_pattern) {
                            count += 1;
                            print_line = true;
                        }
                    }
                    else {
                        if line.to_lowercase().contains(&search_pattern.to_lowercase()) {
                            count += 1;
                            print_line = true;
                        }
                    }
                    
                    if print_line {
                        println!("{}:{} {}", path.blue(), line_number, line);
                    }

                    if count_limit {
                        if count >= max_count {
                            break;
                        }
                    }

                }
                
            }
        }
        else {
            continue;
        }
    }

}


fn main() {
    loop {

        let current_dir = env::current_dir().unwrap();
        let dir_str = format!("{}",current_dir.display()).purple().bold();
        print!("{}: ",dir_str);
        stdout().flush().unwrap();

        let mut input = String::new();
        let _ = stdin().read_line(&mut input).unwrap();

        if input == "\n" {
            continue;
        } 

        let commands = input.split("&&").collect::<Vec<&str>>();

        for cmd in commands {

            let mut parts = cmd.trim().split_whitespace();
            let command = parts.next().unwrap();
            let args = parts.collect::<Vec<&str>>(); 
            
            match command {
                "exit" => return,
                "echo" => echo(args),
                "pwd" => {
                    if args.len()>0 {
                        eprintln!("Did you mean pwd?");
                    }
                    else {
                        println!("{}",current_dir.display())
                    }
                },
                "ls" => ls(args),
                "cat" => {
                    if let Some(filename) = args.get(0) {
                        cat(filename.to_string());
                    }
                },
                "touch" => {
                    if let Some(filename) = args.get(0) {
                        touch(filename.to_string());
                    }
                },
                "mkdir" => {
                    if let Some(dir) = args.get(0) {
                        mkdir(dir.to_string());
                    }
                },
                "cd" => {
                    if let Some(path) = args.get(0) {
                        cd(path);
                    }
                },
                "grep" => {
                    if args.len() >= 2 {
    
                        let mut search_pattern = String::new();
                        let mut paths: Vec<&str> = Vec::new();
                        let mut opts: Vec<&str> = Vec::new();
    
                        let mut execute = true;
    
                        for (i, a) in args.clone().into_iter().enumerate() {
                            if i == 0 {
                                let mut trimmed =  a.trim();
                                if trimmed.starts_with('"') && trimmed.ends_with('"') {
                                    trimmed = &trimmed[1..trimmed.len() - 1];
                                }
    
                                search_pattern = trimmed.to_owned().to_string();
                            }
    
                            match a {
                                "-i" => opts.push(a),
                                "-m" => {
                                    opts.push(a);
                                    if let Some(count_str) = args.get(i+1) {
                                        if let Ok(_count) = count_str.parse::<i32>() {
                                            opts.push(count_str);
                                        } 
                                        else {
                                            eprintln!("Error: Invalid argument for -m option");
                                            execute = false;
                                            break;
                                        }
                                    } 
                                    else {
                                        eprintln!("Error: Missing argument for -m option");
                                        execute = false;
                                        break;
                                    }
                                },
    
                                _ => paths.push(a)
                                 
                            }
    
                        }
                        
                        if execute {
                            grep(search_pattern,paths,opts);
                        }
    
                    }
                    else {
                        eprintln!("Error in `grep` syntax: grep \"search pattern\" [file_name] [options]");
                    }
                },
                
                _ => eprintln!("Error: Command not recognized!")
            }
        }
    }
}