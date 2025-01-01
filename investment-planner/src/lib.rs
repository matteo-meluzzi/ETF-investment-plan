mod calc_etf_items;

use investment_strategy::solve_etf_problem;
use derive_new::new;
use crate::calc_etf_items::calc_etf_items;

pub type EtfId = String;

#[derive(Debug, Clone, Eq, PartialEq, Hash, new)]
pub struct Investment {
    pub etf_id: EtfId,
    pub name: String,
    pub quantity: i64,
    pub price: i64,
}

#[derive(Debug, Clone, PartialEq, new)]
pub struct EtfSetting {
    pub id: EtfId,
    pub isin: String,
    pub name: String,
    pub ideal_proportion: f64,
    pub cumulative: i64
}

#[derive(Debug, Clone, PartialEq, new)]
pub struct Settings {
    pub budget: i64,
    pub etf_settings: Vec<EtfSetting>,
}

pub fn next_investments(settings: Settings, prices: &[f64]) -> Vec<Investment> {
    assert!(prices.iter().all(|&p| p > 0.0));
    let items = calc_etf_items(&settings, prices);
    let solution = solve_etf_problem(settings.budget, items);

    let investments = solution.into_iter()
            .zip(settings.etf_settings)
            .map(|((item, quantity), etf_setting)| Investment::new(etf_setting.id, etf_setting.name, quantity, item.price));
    investments.collect()
}

pub fn total_amount_spent(investments: &[Investment], prices: &[f64]) -> f64 {
    investments.iter().zip(prices).map(|(i, p)| i.quantity as f64 * p).sum()
}

pub fn left_over_budget(budget: i64, investments: &[Investment], prices: &[f64]) -> i64 {
    budget - total_amount_spent(&investments, &prices) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_investments_one() {
        let settings = Settings::new(600_00, vec![EtfSetting::new("ID1".into(), "".to_string(), "".to_string(),0.5, 100_00)]);
        let prices = vec![5_00f64];
        let investments = next_investments(settings, &prices);
        assert_eq!(investments, vec![Investment::new("ID1".into(), "".to_string(), "".to_string(),120)])
    }

    #[test]
    fn test_next_investments_thre() {
        let settings = Settings::new(500_00, vec![
            EtfSetting::new("ID1".into(), "".to_string(), "".to_string(),0.5, 100_00),
            EtfSetting::new("ID2".into(), "".to_string(), "".to_string(),1.0, 100_00),
            EtfSetting::new("ID3".into(), "".to_string(), "".to_string(),0.5, 100_00),
        ]);
        let prices = vec![5_00f64, 5_00f64, 5_00f64];
        let investments = next_investments(settings, &prices);
        assert_eq!(investments, vec![
            Investment::new("ID1".into(), "".to_string(), 20),
            Investment::new("ID2".into(), "".to_string(),60),
            Investment::new("ID3".into(), "".to_string(),20),
        ])
    }

    #[test]
    fn test_next_investments_three_not_perfect() {
        let etf_settings = vec![
            EtfSetting::new("ID1".into(), "".to_string(), "".to_string(),0.25, 25_00),
            EtfSetting::new("ID2".into(), "".to_string(), "".to_string(),0.25, 25_00),
            EtfSetting::new("ID3".into(), "".to_string(), "".to_string(),0.5, 50_00),
        ];
        let settings = Settings::new(100_00, etf_settings.clone());
        let prices = vec![7_00f64, 9_00f64, 3_00f64];
        let investments = next_investments(settings, &prices);
        let final_amounts = investments.iter().zip(prices.iter()).map(|(i, p)| i.quantity as f64 * p).zip(etf_settings).map(|(a, s)| a + s.cumulative as f64).collect::<Vec<_>>();

        assert_eq!(final_amounts, vec![
            46_00.0,
            52_00.0,
            101_00.0
        ]);

        assert_eq!(total_amount_spent(&investments, &prices), 99_00.0);
        assert_eq!(left_over_budget(100_00, &investments, &prices), 1_00);
    }
}

