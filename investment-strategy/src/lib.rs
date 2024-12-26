mod rclist;
mod knap_sack;

use derive_new::new;
use knap_sack::{knap_sack_rc_list, generate_weights_and_values};

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, new)]
pub struct EtfItem {
    pub cumulative: i64,
    pub target: i64,
    pub price: i64,
}

pub fn solve_etf_problem(budget: i64, etfs: Vec<EtfItem>) -> Vec<(EtfItem, i64)> {
    let items = generate_weights_and_values(budget, &etfs);
    let weights = items.iter().map(|item| item.weight).collect::<Vec<_>>();
    let values = items.iter().map(|item| item.value).collect::<Vec<_>>();

    let mut buy_quantities = vec![0i64; etfs.len()];
    let (_, item_indices) = knap_sack_rc_list(budget, &weights, &values);
    for item_index in item_indices {
        let item = items[item_index];
        buy_quantities[item.etf_index] += 1;
    }

    etfs.into_iter().zip(buy_quantities).collect()
}

pub fn calc_total_price(etfs: &[EtfItem], buy_quantities: &[i64]) -> i64 {
    etfs.iter().zip(buy_quantities).map(|(etf, quantity)| etf.price * quantity).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calc_total_error(etfs: &[EtfItem], buy_quantities: &[i64]) -> i64 {
        etfs.iter().zip(buy_quantities).map(|(etf, quantity)| {
            (etf.target - (etf.cumulative + etf.price * quantity)).pow(2)
        }).sum()
    }

    #[test]
    fn test_knap_sack_three() {
        let (value, indices) = knap_sack_rc_list(600, &vec![300, 200, 250], &vec![150, 200, 250]);
        assert_eq!(value, 450);
        assert_eq!(indices, vec![1, 2]);
    }

    #[test]
    fn test_knap_sack_memo_set_three() {
        let (value, indices) = knap_sack_rc_list(600, &vec![300, 200, 250], &vec![150, 200, 250]);
        assert_eq!(value, 450);
        assert_eq!(indices, vec![1, 2]);
    }

    #[test]
    fn test_solve_etf_problem_one() {
        let budget = 10;
        let etfs = vec![
            EtfItem::new(0, 5, 1),
        ];
        let buy_quantities = solve_etf_problem(budget, etfs.clone());
        assert_eq!(buy_quantities, etfs.into_iter().zip(vec![5]).collect::<Vec<_>>());
    }

    #[test]
    fn test_solve_etf_problem_two() {
        let budget = 89;
        let etfs = vec![
            EtfItem::new(0, 5, 8),
            EtfItem::new(80, 95, 5),
        ];
        let buy_quantities = solve_etf_problem(budget, etfs.clone());
        assert_eq!(buy_quantities, etfs.into_iter().zip(vec![1, 3]).collect::<Vec<_>>());
    }

    #[test]
    fn test_solve_etf_problem_three() {
        let budget = 600;
        let etfs = vec![
            EtfItem::new(0, 200, 300),
            EtfItem::new(0, 200, 200),
            EtfItem::new(0, 200, 250),
        ];
        let buy_quantities = solve_etf_problem(budget, etfs.clone());
        assert_eq!(buy_quantities, etfs.clone().into_iter().zip(vec![0, 1, 1]).collect::<Vec<_>>());
        let total_price = calc_total_price(&etfs, &buy_quantities.iter().map(|q| q.1).collect::<Vec<_>>());
        assert_eq!(total_price, 450);
        let total_error = calc_total_error(&etfs, &buy_quantities.iter().map(|q| q.1).collect::<Vec<_>>());
        assert_eq!(total_error, 42500);
    }

    #[test]
    fn test_solve_etf_problem_threee() {
        let budget = 600-450 + 600;
        let etfs = vec![
            EtfItem::new(0, 400, 300),
            EtfItem::new(200, 400, 200),
            EtfItem::new(250, 400, 250),
        ];
        let buy_quantities = solve_etf_problem(budget, etfs.clone());
        assert_eq!(buy_quantities, etfs.clone().into_iter().zip(vec![1, 1, 1]).collect::<Vec<_>>());
        let total_price = calc_total_price(&etfs, &buy_quantities.iter().map(|q| q.1).collect::<Vec<_>>());
        assert_eq!(total_price, 750);
        let total_error = calc_total_error(&etfs, &buy_quantities.iter().map(|q| q.1).collect::<Vec<_>>());
        assert_eq!(total_error, 20000);
    }

    #[test]
    fn test_solve_etf_problem_threeee() {
        let budget = 600 - 750 + 600-450 + 600;
        let etfs = vec![
            EtfItem::new(300, 600, 300),
            EtfItem::new(400, 600, 200),
            EtfItem::new(500, 600, 250),
        ];
        let buy_quantities = solve_etf_problem(budget, etfs.clone());
        assert_eq!(buy_quantities, etfs.clone().into_iter().zip(vec![1, 1, 0]).collect::<Vec<_>>());
        let total_price = calc_total_price(&etfs, &buy_quantities.iter().map(|q| q.1).collect::<Vec<_>>());
        assert_eq!(total_price, 500);
        let total_error = calc_total_error(&etfs, &buy_quantities.iter().map(|q| q.1).collect::<Vec<_>>());
        assert_eq!(total_error, 10000);
    }

    #[test]
    fn test_solve_etf_problem_threeeee() {
        let budget = 600 - 500 + 600 - 750 + 600 - 450 + 600;
        let etfs = vec![
            EtfItem::new(600, 800, 300),
            EtfItem::new(600, 800, 200),
            EtfItem::new(500, 800, 250),
        ];
        let buy_quantities = solve_etf_problem(budget, etfs.clone());
        assert_eq!(buy_quantities, etfs.clone().into_iter().zip(vec![0, 1, 1]).collect::<Vec<_>>());
        let total_price = calc_total_price(&etfs, &buy_quantities.iter().map(|q| q.1).collect::<Vec<_>>());
        assert_eq!(total_price, 450);
        let total_error = calc_total_error(&etfs, &buy_quantities.iter().map(|q| q.1).collect::<Vec<_>>());
        assert_eq!(total_error, 42500);
    }

    #[test]
    fn test_solve_etf_problem_three_small_budget() {
        let budget = 150;
        let etfs = vec![
            EtfItem::new(600, 800, 300),
            EtfItem::new(600, 800, 200),
            EtfItem::new(500, 800, 250),
        ];
        let buy_quantities = solve_etf_problem(budget, etfs.clone());
        assert_eq!(buy_quantities,etfs.into_iter().zip(vec![0, 0, 0]).collect::<Vec<_>>());
    }

    #[test]
    fn test_solve_etf_problem_three_negative_target() {
        let budget = 850;
        let etfs = vec![
            EtfItem::new(600, 800, 300),
            EtfItem::new(600, 500, 200),
            EtfItem::new(500, 1000, 250),
        ];
        let buy_quantities = solve_etf_problem(budget, etfs.clone());
        assert_eq!(buy_quantities, etfs.into_iter().zip(vec![1, 0, 2]).collect::<Vec<_>>());
    }

    #[test]
    fn test_generate_weights_and_values_one() {
        let budget = 10;
        let etfs = vec![
            EtfItem::new(0, 5, 1),
        ];
        let items = generate_weights_and_values(budget, &etfs);
        assert_eq!(items.len(), 5);
        let values = items.iter().map(|item| item.value).collect::<Vec<_>>();
        assert_eq!(values, vec![9, 7, 5, 3, 1]);
        let weights = items.iter().map(|item| item.weight).collect::<Vec<_>>();
        assert_eq!(weights, vec![1; 5]);
    }

    #[test]
    fn test_knap_sack_with_no_items() {
        let weights = vec![];
        let values = vec![];
        let max_weight = 10;

        let (max_value, selected_items) = knap_sack_rc_list(max_weight, &weights, &values);

        // No items, so max_value should be 0, and the selected_items set should be empty.
        assert_eq!(max_value, 0);
        assert!(selected_items.is_empty());
    }

    #[test]
    fn test_knap_sack_with_no_capacity() {
        let weights = vec![5, 10, 15];
        let values = vec![10, 20, 30];
        let max_weight = 0;

        let (max_value, selected_items) = knap_sack_rc_list(max_weight, &weights, &values);

        // No capacity, so the max_value should be 0, and no items can be selected.
        assert_eq!(max_value, 0);
        assert!(selected_items.is_empty());
    }

    #[test]
    fn test_knap_sack_with_exact_capacity() {
        let weights = vec![5, 10, 15];
        let values = vec![10, 19, 30];
        let max_weight = 15;

        let (max_value, selected_items) = knap_sack_rc_list(max_weight, &weights, &values);

        // Max weight is exactly the weight of the third item, so we expect it to be selected.
        assert_eq!(max_value, 30);
        let expected_items = vec![2];
        assert_eq!(selected_items, expected_items);
    }

    #[test]
    fn test_knap_sack_with_multiple_choices() {
        let weights = vec![5, 10, 15];
        let values = vec![10, 20, 30];
        let max_weight = 20;

        let (max_value, selected_items) = knap_sack_rc_list(max_weight, &weights, &values);

        // The best value comes from selecting the first and third items, total weight = 20.
        assert_eq!(max_value, 40);
        let expected_items = vec![0, 2];
        assert_eq!(selected_items, expected_items);
    }

    #[test]
    fn test_knap_sack_with_large_weights_and_values() {
        let weights = vec![1, 3, 4, 5];
        let values = vec![1, 4, 5, 7];
        let max_weight = 7;

        let (max_value, selected_items) = knap_sack_rc_list(max_weight, &weights, &values);

        // The optimal selection is the items with weights 3 and 4, values 4 and 5, total value = 9.
        assert_eq!(max_value, 9);
        let expected_items = vec![1, 2];
        assert_eq!(selected_items, expected_items);
    }

    #[test]
    fn test_knap_sack_no_items_fit() {
        let weights = vec![10, 20, 30];
        let values = vec![60, 100, 120];
        let max_weight = 5;

        let (max_value, selected_items) = knap_sack_rc_list(max_weight, &weights, &values);

        // No items fit in the knapsack, so the value should be 0 and no items selected.
        assert_eq!(max_value, 0);
        assert!(selected_items.is_empty());
    }
}
