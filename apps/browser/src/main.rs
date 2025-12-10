use lucastra_browser::{Browser, HttpClient};
use std::io::{self, Write};

fn main() {
    println!("LucAstra Lightweight Browser v0.1.0");
    println!("Commands: open <url>, tab <url>, close, back, bookmark, bookmarks, tabs, exit");
    println!("Example: open https://www.example.com\n");

    let client = HttpClient::new();
    let mut browser = Browser::new();

    loop {
        print!("> ");
        io::stdout().flush().ok();

        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();

        let input = input.trim();
        let parts: Vec<&str> = input.split_whitespace().collect();

        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
            println!("Goodbye!");
            break;
        }

        if input.eq_ignore_ascii_case("back") {
            match browser.back(&client) {
                Ok(_) => {
                    if let Some(tab) = browser.current_tab() {
                        println!("Back to: {}", tab.url);
                    }
                }
                Err(e) => println!("Error: {}", e),
            }
            continue;
        }

        if input.eq_ignore_ascii_case("tabs") {
            for (i, tab) in browser.tabs.iter().enumerate() {
                let marker = if i == browser.active_tab { "*" } else { " " };
                println!("[{}] {} {}", marker, i, tab.url);
            }
            continue;
        }

        if input.eq_ignore_ascii_case("bookmark") {
            browser.bookmark();
            if let Some(tab) = browser.current_tab() {
                println!("Bookmarked: {}", tab.url);
            }
            continue;
        }

        if input.eq_ignore_ascii_case("bookmarks") {
            if browser.bookmarks.is_empty() {
                println!("No bookmarks");
            } else {
                for (i, url) in browser.bookmarks.iter().enumerate() {
                    println!("[{}] {}", i, url);
                }
            }
            continue;
        }

        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "open" => {
                if parts.len() > 1 {
                    let url = parts[1..].join(" ");
                    println!("Loading {}...", url);
                    match browser.navigate(url.clone(), &client) {
                        Ok(_) => {
                            if let Some(tab) = browser.current_tab() {
                                if let Some(content) = &tab.content {
                                    println!("\n=== {} ===\n", content.title);
                                    println!("{}\n", content.text);

                                    if !content.links.is_empty() {
                                        println!("\nLinks:");
                                        for (i, link) in content.links.iter().enumerate().take(10) {
                                            println!("[{}] {} ({})", i, link.text, link.href);
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                } else {
                    println!("Usage: open <url>");
                }
            }
            "tab" => {
                if parts.len() > 1 {
                    let url = parts[1..].join(" ");
                    browser.new_tab(url);
                    println!("New tab opened (tab index: {})", browser.active_tab);
                } else {
                    println!("Usage: tab <url>");
                }
            }
            "close" => {
                if browser.tabs.len() > 1 {
                    browser.close_tab();
                    println!("Tab closed");
                } else {
                    println!("Cannot close the last tab");
                }
            }
            _ => println!("Unknown command. Try: open, tab, close, back, bookmark, bookmarks, tabs, exit"),
        }
    }
}
