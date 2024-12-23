use std::{collections::HashSet, rc::Rc};
use derive_new::new;

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, new)]
pub struct EtfItem {
    cumulative: i64,
    target: i64,
    price: i64,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
enum RcList<T> {
    Node(T, Rc<RcList<T>>),
    Stop
}
impl<T: Clone> IntoIterator for RcList<T> {
    type Item = T;
    type IntoIter = RcListIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        RcListIterator { current_list: Rc::new(self) }
    }
}

struct RcListIterator<T> {
    current_list: Rc<RcList<T>>
}
impl<T: Clone> Iterator for RcListIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let curr = self.current_list.clone();
        match curr.as_ref() {
            RcList::Stop => None,
            RcList::Node(t, next) => {
                self.current_list = next.clone();
                Some(t.clone())
            }
        }
    }
}

fn knap_sack(max_weight: i64, weights: &[i64], values: &[i64]) -> (i64, HashSet<usize>) {
    let max_weight = max_weight as usize;
    let weights = weights.iter().map(|w| *w as usize).collect::<Vec<_>>();

    let mut dp = vec![0; max_weight + 1];
    let mut sets = vec![HashSet::<usize>::new(); max_weight + 1];

    for i in 1..=weights.len() {
        for w in (0..=max_weight).rev() {
            if weights[i - 1] <= w {
                if dp[w] < dp[w - weights[i - 1]] + values[i - 1] {
                    sets[w] = sets[w - weights[i - 1]].clone();
                    sets[w].insert(i - 1);

                    dp[w] = dp[w - weights[i - 1]] + values[i - 1]
                }
            }
        }
    }

    (dp[max_weight], sets[max_weight].clone())
}

fn knap_sack_tree(max_weight: i64, weights: &[i64], values: &[i64]) -> (i64, Vec<usize>) {
    let max_weight = max_weight as usize;
    let weights = weights.iter().map(|w| *w as usize).collect::<Vec<_>>();

    let mut dp = vec![0; max_weight + 1];
    let mut trees = vec![Rc::new(RcList::Stop); max_weight + 1];

    for i in 1..=weights.len() {
        for w in (0..=max_weight).rev() {
            if weights[i - 1] <= w {
                if dp[w] < dp[w - weights[i - 1]] + values[i - 1] {
                    assert_ne!(weights[i-1], 0);
                    let tree = RcList::Node(i - 1, trees[w - weights[i-1]].clone());
                    trees[w] = Rc::new(tree);

                    dp[w] = dp[w - weights[i - 1]] + values[i - 1]
                }
            }
        }
    }

    (dp[max_weight], Vec::from_iter(trees[max_weight].as_ref().clone().into_iter()))
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, new)]
struct KnapSackItem {
    value: i64,
    weight: i64,

    etf_index: usize,
}

fn generate_weights_and_values(budget: i64, etfs: &[EtfItem]) -> Vec<KnapSackItem> {
    let mut items = vec![];

    for (etf_index, etf) in etfs.iter().enumerate() {
        let mut buy_quantity = 1;
        let mut last_error = (etf.target - etf.cumulative).pow(2);
        while etf.price * buy_quantity <= budget {
            let amount = etf.cumulative + (etf.price * buy_quantity);
            let error = (etf.target - amount).pow(2);
            let value = last_error - error;
            if value <= 0 {
                break;
            }

            items.push(KnapSackItem::new(value, etf.price, etf_index));

            last_error = error;
            buy_quantity += 1;
        }
    }
    
    items
}

pub fn solve_etf_problem(budget: i64, etfs: &[EtfItem]) -> Vec<i64> {
    let items = generate_weights_and_values(budget, etfs);
    let weights = items.iter().map(|item| item.weight).collect::<Vec<_>>();
    let values = items.iter().map(|item| item.value).collect::<Vec<_>>();

    let mut buy_quantities = vec![0i64; etfs.len()];
    let (_, item_indices) = knap_sack_tree(budget, &weights, &values);
    for item_index in item_indices {
        let item = items[item_index];
        buy_quantities[item.etf_index] += 1;
    }

    buy_quantities
}

pub fn calc_total_price(etfs: &[EtfItem], buy_quantities: &[i64]) -> i64 {
    etfs.iter().zip(buy_quantities).map(|(etf, quantity)| etf.price * quantity).sum()
}

pub fn calc_total_error(etfs: &[EtfItem], buy_quantities: &[i64]) -> i64 {
    etfs.iter().zip(buy_quantities).map(|(etf, quantity)| {
        (etf.target - (etf.cumulative + etf.price * quantity)).pow(2)
    }).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knap_sack_three() {
        let (value, indices) = knap_sack(600, &vec![300, 200, 250], &vec![150, 200, 250]);
        assert_eq!(value, 450);
        assert_eq!(indices, vec![1, 2].into_iter().collect());
    }

    #[test]
    fn test_knap_sack_memo_set_three() {
        let (value, indices) = knap_sack(600, &vec![300, 200, 250], &vec![150, 200, 250]);
        assert_eq!(value, 450);
        assert_eq!(indices, vec![1, 2].into_iter().collect());
    }

    #[test]
    fn test_solve_etf_problem_one() {
        let budget = 10;
        let etfs = vec![
            EtfItem::new(0, 5, 1),
        ];
        let buy_quantities = solve_etf_problem(budget, &etfs);
        assert_eq!(buy_quantities, vec![5]);
    }

    #[test]
    fn test_solve_etf_problem_two() {
        let budget = 89;
        let etfs = vec![
            EtfItem::new(0, 5, 8),
            EtfItem::new(80, 95, 5),
        ];
        let buy_quantities = solve_etf_problem(budget, &etfs);
        assert_eq!(buy_quantities, vec![1, 3]);
    }

    #[test]
    fn test_solve_etf_problem_three() {
        let budget = 600;
        let etfs = vec![
            EtfItem::new(0, 200, 300),
            EtfItem::new(0, 200, 200),
            EtfItem::new(0, 200, 250),
        ];
        let buy_quantities = solve_etf_problem(budget, &etfs);
        assert_eq!(buy_quantities, vec![0, 1, 1]);
        let total_price = calc_total_price(&etfs, &buy_quantities);
        assert_eq!(total_price, 450);
        let total_error = calc_total_error(&etfs, &buy_quantities);
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
        let buy_quantities = solve_etf_problem(budget, &etfs);
        assert_eq!(buy_quantities, vec![1, 1, 1]);
        let total_price = calc_total_price(&etfs, &buy_quantities);
        assert_eq!(total_price, 750);
        let total_error = calc_total_error(&etfs, &buy_quantities);
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
        let buy_quantities = solve_etf_problem(budget, &etfs);
        assert_eq!(buy_quantities, vec![1, 1, 0]);
        let total_price = calc_total_price(&etfs, &buy_quantities);
        assert_eq!(total_price, 500);
        let total_error = calc_total_error(&etfs, &buy_quantities);
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
        let buy_quantities = solve_etf_problem(budget, &etfs);
        assert_eq!(buy_quantities, vec![0, 1, 1]);
        let total_price = calc_total_price(&etfs, &buy_quantities);
        assert_eq!(total_price, 450);
        let total_error = calc_total_error(&etfs, &buy_quantities);
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
        let buy_quantities = solve_etf_problem(budget, &etfs);
        assert_eq!(buy_quantities, vec![0, 0, 0]);
    }

    #[test]
    fn test_solve_etf_problem_three_negative_target() {
        let budget = 850;
        let etfs = vec![
            EtfItem::new(600, 800, 300),
            EtfItem::new(600, 500, 200),
            EtfItem::new(500, 1000, 250),
        ];
        let buy_quantities = solve_etf_problem(budget, &etfs);
        assert_eq!(buy_quantities, vec![1, 0, 2]);
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

        let (max_value, selected_items) = knap_sack(max_weight, &weights, &values);

        // No items, so max_value should be 0, and the selected_items set should be empty.
        assert_eq!(max_value, 0);
        assert!(selected_items.is_empty());
    }

    #[test]
    fn test_knap_sack_with_no_capacity() {
        let weights = vec![5, 10, 15];
        let values = vec![10, 20, 30];
        let max_weight = 0;

        let (max_value, selected_items) = knap_sack(max_weight, &weights, &values);

        // No capacity, so the max_value should be 0, and no items can be selected.
        assert_eq!(max_value, 0);
        assert!(selected_items.is_empty());
    }

    #[test]
    fn test_knap_sack_with_exact_capacity() {
        let weights = vec![5, 10, 15];
        let values = vec![10, 19, 30];
        let max_weight = 15;

        let (max_value, selected_items) = knap_sack(max_weight, &weights, &values);

        // Max weight is exactly the weight of the third item, so we expect it to be selected.
        assert_eq!(max_value, 30);
        let expected_items = vec![2].into_iter().collect::<HashSet<_>>();
        assert_eq!(selected_items, expected_items);
    }

    #[test]
    fn test_knap_sack_with_multiple_choices() {
        let weights = vec![5, 10, 15];
        let values = vec![10, 20, 30];
        let max_weight = 20;

        let (max_value, selected_items) = knap_sack(max_weight, &weights, &values);

        // The best value comes from selecting the first and third items, total weight = 20.
        assert_eq!(max_value, 40);
        let expected_items = vec![0, 2].into_iter().collect::<HashSet<_>>();
        assert_eq!(selected_items, expected_items);
    }

    #[test]
    fn test_knap_sack_with_large_weights_and_values() {
        let weights = vec![1, 3, 4, 5];
        let values = vec![1, 4, 5, 7];
        let max_weight = 7;

        let (max_value, selected_items) = knap_sack(max_weight, &weights, &values);

        // The optimal selection is the items with weights 3 and 4, values 4 and 5, total value = 9.
        assert_eq!(max_value, 9);
        let expected_items = vec![1, 2].into_iter().collect::<HashSet<_>>();
        assert_eq!(selected_items, expected_items);
    }

    #[test]
    fn test_knap_sack_no_items_fit() {
        let weights = vec![10, 20, 30];
        let values = vec![60, 100, 120];
        let max_weight = 5;

        let (max_value, selected_items) = knap_sack(max_weight, &weights, &values);

        // No items fit in the knapsack, so the value should be 0 and no items selected.
        assert_eq!(max_value, 0);
        assert!(selected_items.is_empty());
    }
}
