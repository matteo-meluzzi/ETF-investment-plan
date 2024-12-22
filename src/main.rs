use std::io::Write;
use yahoo_finance_info::ETF;

fn main() {


    print!("Please enter a quote name: ");
    std::io::stdout().lock().flush().unwrap();
    let mut quote_name = String::new();
    std::io::stdin().read_line(&mut quote_name).unwrap();
    let quote_name = quote_name.trim();
    let quote = get_quote(&quote_name).unwrap();
    println!("Most recent price of {quote_name} is {quote}");
}

// IWDA.L -> ishares world