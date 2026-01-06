use lucastra_file_manager::FileManager;
use std::path::PathBuf;

fn main() {
    let start_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    println!("LucAstra File Manager v0.1.0");
    println!("Starting at: {}\n", start_dir.display());

    match FileManager::new(start_dir) {
        Ok(mut fm) => loop {
            print_entries(&fm);
            print!("> ");
            use std::io::{self, Write};
            io::stdout().flush().ok();

            let mut input = String::new();
            io::stdin().read_line(&mut input).ok();
            let input = input.trim();

            if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                println!("Goodbye!");
                break;
            }

            if input.eq_ignore_ascii_case("ls") || input.eq_ignore_ascii_case("list") {
                println!("(list displayed above)");
                continue;
            }

            if input.eq_ignore_ascii_case("back") || input == ".." {
                match fm.back() {
                    Ok(_) => println!("Back"),
                    Err(e) => println!("Error: {}", e),
                }
                continue;
            }

            if let Some(path) = input.strip_prefix("cd ") {
                match fm.navigate(PathBuf::from(path).as_path()) {
                    Ok(_) => println!("Directory changed"),
                    Err(e) => println!("Error: {}", e),
                }
                continue;
            }

            if let Some(path) = input.strip_prefix("rm ") {
                match fm.delete(PathBuf::from(path).as_path()) {
                    Ok(_) => println!("Deleted"),
                    Err(e) => println!("Error: {}", e),
                }
                continue;
            }

            if let Some(stripped) = input.strip_prefix("cp ") {
                let parts: Vec<&str> = stripped.split(' ').collect();
                if parts.len() == 2 {
                    match fm.copy(
                        PathBuf::from(parts[0]).as_path(),
                        PathBuf::from(parts[1]).as_path(),
                    ) {
                        Ok(_) => println!("Copied"),
                        Err(e) => println!("Error: {}", e),
                    }
                } else {
                    println!("Usage: cp <source> <dest>");
                }
                continue;
            }

            if let Ok(index) = input.parse::<usize>() {
                if let Some(path) = fm.get_path(index) {
                    if path.is_dir() {
                        match fm.navigate(&path) {
                            Ok(_) => println!("Entered directory"),
                            Err(e) => println!("Error: {}", e),
                        }
                    } else {
                        println!("File: {}", path.display());
                    }
                } else {
                    println!("Invalid index");
                }
                continue;
            }

            println!("Unknown command. Try: ls, cd <path>, cp <src> <dest>, rm <path>, <index>, back, exit");
        },
        Err(e) => println!("Error: {}", e),
    }
}

fn print_entries(fm: &FileManager) {
    println!("\nDirectory: {}\n", fm.current_dir.display());
    for (i, entry) in fm.list().iter().enumerate() {
        let marker = if entry.is_dir { "/" } else { "" };
        println!("[{}] {} {} ({} bytes)", i, entry.name, marker, entry.size);
    }
    println!();
}
