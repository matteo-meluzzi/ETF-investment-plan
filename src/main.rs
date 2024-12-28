use std::io::{self, BufRead};
use std::str::FromStr;
use futures::future::try_join_all;
use yahoo_finance_info::{get_price_of, search_etf_isin, YahooError};
use investment_planner::{next_investments, EtfSetting, Settings};
use tokio;

fn ask_until_valid<T: FromStr, R: BufRead>(reader: &mut R, err_msg: &str, constraint: Option<&dyn Fn(&T) -> bool>) -> Result<T, io::Error> {
    let result: T;
    loop {
        let mut line = "".to_string();
        reader.read_line(&mut line)?;
        match (line.trim().parse::<T>(), constraint) {
            (Ok(n), Some(constraint)) if constraint(&n) => {
                result = n;
                break;
            }
            (Ok(n), None) => {
                result = n;
                break;
            }
            _ => {
                println!("{}", err_msg);
            }
        }
    }
    Ok(result)
}

#[derive(Debug)]
enum PlannerError {
    IOError(io::Error),
    YahooError(YahooError),
}
impl From<io::Error> for PlannerError {
    fn from(err: io::Error) -> PlannerError { PlannerError::IOError(err) }
}
impl From<YahooError> for PlannerError {
    fn from(err: YahooError) -> PlannerError { PlannerError::YahooError(err) }
}

async fn ask_settings<R: BufRead>(mut reader: R) -> Result<Settings, PlannerError> {
    println!("What's your budget? (whole euros) ");
    let budget = ask_until_valid::<i64, R>(&mut reader, "Please enter a valid integer number: ", None)?;

    let mut etf_settings = vec![];
    loop {
        println!("Enter an etf ISIN or 'stop' to stop:");
        let isin = ask_until_valid::<String, R>(&mut reader,"Please enter a valid string: ", None)?;
        if isin == "stop" { break; }

        let mut search_results = yahoo_finance_info::search_etf_isin(&isin).await?;
        if search_results.is_empty() {
            println!("Searching for {isin} did not produce any results.");
            continue;
        }
        let index = if search_results.len() == 1 {
            0
        } else {
            println!("Which ETF did you mean? Enter the number of the ETF you meant:");
            for (i, search_result) in search_results.iter().enumerate() {
                println!("{}: {} {}", i, search_result.name, search_result.isin);
            }
            ask_until_valid::<usize, R>(&mut reader, "Please enter a valid index: ", Some(&|&index| index < search_results.len()))?
        };
        let chosen_etf = search_results.swap_remove(index);
        let id = chosen_etf.ticker;
        let name = chosen_etf.name;

        println!("What is the ideal proportion of {name} in your portfolio? ");
        let prop = ask_until_valid::<f64, R>(&mut reader,"Please enter a valid float: ", None)?;

        println!("How much of {name} do you own already? ");
        let cumulative = ask_until_valid::<f64, R>(&mut reader, "Please enter a valid float: ", None)?;

        etf_settings.push(EtfSetting::new(id, name, prop, (cumulative * 100.0) as i64));
    }

    Ok(Settings::new(budget, etf_settings))
}

fn suggest_investments(settings: Settings, prices: &[f64]) {
    let cumulatives = settings.etf_settings.iter().map(|e| e.cumulative).collect::<Vec<_>>();
    let total_portfolio = cumulatives.iter().sum::<i64>() as f64;
    let investments = next_investments(settings, prices);
    let total_investments = investments.iter().zip(prices).map(|(i, price)| i.quantity as f64 * price).sum::<f64>();
    let proportions = investments
        .iter()
        .zip(prices)
        .zip(cumulatives)
        .map(|((i, price), amount)| (i.quantity as f64 * price + amount as f64) / (total_portfolio + total_investments))
        .collect::<Vec<_>>();
    for ((investment, price), proportion) in investments.into_iter().zip(prices).zip(proportions) {
        println!("{} buy {} for {:.2} real proportion: {:.2}", investment.name, investment.quantity, price, proportion);
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // let settings = ask_settings(io::stdin().lock()).await.unwrap();
    let settings = Settings::new(500, vec![
        EtfSetting::new("AGGG.L".into(), "iShares Core Global Aggregate Bond UCITS ETF USD (Dist)".into(), 0.9, 100),
        EtfSetting::new("IUIT.L".into(), "iShares S&P 500 Information Technology Sector UCITS ETF USD (Acc)".into(), 0.1, 50),
    ]);
    let prices = try_join_all(settings.etf_settings.iter().map(|etf| get_price_of(&etf.id))).await.unwrap();
    suggest_investments(settings, &prices);
}