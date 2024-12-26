use yahoo_finance_api as yahoo;
pub use yahoo_finance_api::YahooError;

pub type Isin = String;
pub type Ticker = String;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ETF {
    pub name: String,
    pub isin: Isin,
    pub ticker: Ticker
}

impl ETF {
    pub fn new(name: String, isin: Isin, ticker: Ticker) -> Self {
        Self { name, isin, ticker }
    }
}

pub async fn search_etf_isin(isin: &Isin) -> Result<Vec<ETF>, YahooError> {
    let provider = yahoo::YahooConnector::new()?;
    let resp = provider.search_ticker(&isin).await?;

    Ok(resp.quotes.into_iter().map(|quote| {
        ETF::new(quote.long_name, isin.clone(), quote.symbol)
    }).collect())
}

pub async fn get_price_of(ticker: &Ticker) -> Result<f64, YahooError> {
    let provider = yahoo::YahooConnector::new()?;
    let response = provider.get_latest_quotes(&ticker, "1d").await?;
    let quote = response.last_quote()?;
    Ok(quote.close)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search_etf_isin() {
        let xs = search_etf_isin(&"IE00B3ZW0K18".to_string()).await.unwrap();
        assert_eq!(xs.len(), 1);
        assert_eq!(xs[0], ETF::new("iShares S&P 500 EUR Hedged UCITS ETF (Acc)".to_string(), "IE00B3ZW0K18".into(), "IUSE.L".into()));
    }

    #[tokio::test]
    async fn test_get_price_of() {
        let price = get_price_of(&"IUSE.L".to_string()).await.unwrap();
        assert!(price > 0.0);
    }
}
