extern crate serde;
extern crate serde_json;

mod displayers;
mod matcher;
#[cfg(test)]
mod tests;

use crate::matcher::*;

fn main() {
    let data = include_str!("../requests.json");
    let requests: Vec<Request> = serde_json::from_str(data).expect("was expecting valid JSON file");
    let mut order_book = OrderBook::default();
    let results_iterator = requests
        .iter()
        .map(|request| (request, order_book.match_request(request)));
    for (request, result) in results_iterator {
        println!("{}", request);
        println!("{}\n", result);
    }
}
