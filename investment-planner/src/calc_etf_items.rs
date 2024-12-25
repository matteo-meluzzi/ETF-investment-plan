use investment_strategy::EtfItem;
use crate::Settings;

fn normalize(mut xs: Vec<f64>) -> Vec<f64> {
    let sum: f64 = xs.iter().sum();
    xs.iter_mut().for_each(|x| *x = *x / sum);
    xs
}

fn mult_constant(c: f64, mut xs: Vec<f64>) -> Vec<f64> {
    xs.iter_mut().for_each(|x| *x = *x * c);
    xs
}

fn sum(mut xs: Vec<f64>, ys: &[f64]) -> Vec<f64> {
    xs.iter_mut().zip(ys).for_each(|(x, y)| *x += y);
    xs
}

fn calc_targets(ideal_proportions: Vec<f64>, amounts: &[f64], budget: f64) -> Vec<f64> {
    assert_eq!(ideal_proportions.len(), amounts.len());
    if ideal_proportions.iter().sum::<f64>() <= 0.0 {
        return amounts.iter().map(|a| a + budget / amounts.len() as f64).collect();
    }

    let ideal_proportions = normalize(ideal_proportions);

    let total_amount = amounts.iter().sum::<f64>() + budget;
    let ideal_amounts = ideal_proportions.iter().map(|prop| prop * total_amount);

    let direction = ideal_amounts.zip(amounts).map(|(ideal, real)| (ideal - real).max(0.0)).collect::<Vec<f64>>();
    let direction = normalize(direction);
    let direction = mult_constant(budget, direction);
    sum(direction, amounts)
}

pub fn calc_etf_items(settings: &Settings, prices: &[f64]) -> Vec<EtfItem> {
    let ideal_proportions = settings.etf_settings.iter().map(|etf| etf.ideal_proportion).collect::<Vec<_>>();
    let amounts = settings.etf_settings.iter().map(|etf| etf.cumulative).collect::<Vec<_>>();

    let targets = calc_targets(ideal_proportions, &amounts.iter().map(|&a| a as f64).collect::<Vec<f64>>(), settings.budget as f64);

    amounts.iter()
        .zip(targets)
        .zip(prices)
        .map(|((&cumulative, target), &price)| EtfItem::new(cumulative, target as i64, price as i64))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_target() {
        let ideal_proportions = vec![1.0/3.0, 1.0/3.0, 1.0/3.0];
        let amounts = vec![0.0, 0.0, 1.0];
        let res = calc_targets(ideal_proportions, &amounts, 2.0);
        assert_eq!(res, vec![1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_calc_target_same() {
        let ideal_proportions = vec![0.2, 0.3, 0.1, 0.4];
        let amounts = ideal_proportions.clone();
        let res = calc_targets(ideal_proportions.clone(), &amounts, 1.0);
        assert_eq!(res, mult_constant(2.0, ideal_proportions));
    }

    #[test]
    fn test_calc_target_all_negative() {
        let ideal_proportions = vec![0.0, 0.0];
        let amounts = vec![0.5, 0.5];
        let res = calc_targets(ideal_proportions, &amounts, 1.0);
        assert_eq!(res, vec![1.0, 1.0]);
    }

    #[test]
    fn test_calc_target_all_negative_sum() {
        let ideal_proportions = vec![-1.0, -1.0];
        let amounts = vec![0.5, 0.5];
        let res = calc_targets(ideal_proportions, &amounts, 1.0);
        assert_eq!(res, vec![1.0, 1.0]);
    }

    #[test]
    fn test_calc_target_one_negative_sum() {
        let ideal_proportions = vec![-1.0, 0.1];
        let amounts = vec![0.5, 0.5];
        let res = calc_targets(ideal_proportions, &amounts, 1.0);
        assert_eq!(res, vec![1.0, 1.0]);
    }

    #[test]
    fn test_calc_target_all_positive() {
        let ideal_proportions = vec![0.75, 0.25];
        let amounts = vec![0.0, 0.0];
        let res = calc_targets(ideal_proportions, &amounts, 1.0);
        assert_eq!(res, vec![0.75, 0.25]);
    }

    #[test]
    fn test_calc_target_random_proportions() {
        let ideal_proportions = vec![10.0, 5.0];
        let amounts = vec![5.0, 10.0];
        let res = calc_targets(ideal_proportions, &amounts, 15.0);
        assert_eq!(res, vec![20.0, 10.0]);
    }

    #[test]
    fn test_calc_target_one_zero_proportions() {
        let ideal_proportions = vec![10.0, 0.0];
        let amounts = vec![5.0, 10.0];
        let res = calc_targets(ideal_proportions, &amounts, 150.0);
        assert_eq!(res, vec![155.0, 10.0]);
    }
}
