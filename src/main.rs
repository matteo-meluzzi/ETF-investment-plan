use std::io::Write;
use yahoo_finance_info::ETF;
use investment_strategy::{EtfItem, solve_etf_problem, calc_total_error, calc_total_price};
use std::time::Instant;

fn test_bench_etf() {
    let start = Instant::now();

    const BUDGET: i64 = 2000_00;

    let budget = BUDGET;
    let etfs = vec![
        EtfItem::new(0, BUDGET/5, 10_00),
        EtfItem::new(0, BUDGET/5, 10_00),
        EtfItem::new(0, BUDGET/5, 10_00),
        EtfItem::new(0, BUDGET/5, 10_00),
        EtfItem::new(0, BUDGET/5, 10_00),
    ];
    let buy_quantities = solve_etf_problem(budget, &etfs);
    let total_price = calc_total_price(&etfs, &buy_quantities);
    assert_eq!(total_price, BUDGET);
    let total_error = calc_total_error(&etfs, &buy_quantities);
    println!("total_error: {}", total_error);
    assert_eq!(total_error, 0);

    let duration = start.elapsed();    
    println!("Time taken: {:?}", duration);
}

fn main() {
    test_bench_etf();
}
