use lucastra_calculator::Calculator;

fn main() {
    let mut calc = Calculator::new();

    println!("LucAstra Calculator v0.1.0");
    println!("Type expressions like '2 + 3 * 4' or 'sin 0.5'");
    println!("Type 'exit' to quit\n");

    loop {
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

        if input.eq_ignore_ascii_case("clear") {
            calc.clear();
            println!("Display: {}", calc.display);
            continue;
        }

        if input.eq_ignore_ascii_case("history") {
            if calc.history().is_empty() {
                println!("No history");
            } else {
                for (i, entry) in calc.history().iter().enumerate() {
                    println!("[{}] {}", i, entry);
                }
            }
            continue;
        }

        match calc.eval(input) {
            Ok(result) => println!("Result: {}", result),
            Err(e) => println!("Error: {}", e),
        }
    }
}
