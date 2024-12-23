use std::rc::Rc;
use derive_new::new;

use crate::rclist::RcList;
use crate::EtfItem;

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, new)]
pub struct KnapSackItem {
    pub value: i64,
    pub weight: i64,

    pub etf_index: usize,
}

pub fn generate_weights_and_values(budget: i64, etfs: &[EtfItem]) -> Vec<KnapSackItem> {
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

pub fn knap_sack_rc_list(max_weight: i64, weights: &[i64], values: &[i64]) -> (i64, Vec<usize>) {
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

    let mut indices = Vec::from_iter(trees[max_weight].as_ref().iter().copied());
    indices.reverse();
    assert!(indices.is_sorted());
    (dp[max_weight], indices)
}
