extern crate failure;
extern crate serde_json;
extern crate serde;

use serde::{Deserialize, Serialize};
use failure::Error;

#[derive(Deserialize, Serialize, Debug)]
enum Side {
    Buy,
    Sell
}

#[derive(Deserialize, Serialize, Debug)]
enum Type {
    Limit,
    FillOrKill,
    ImmediateOrCancel
}

#[derive(Deserialize, Serialize, Debug)]
struct Request {
    side: Side,
    price: u64,
    size: u64,
    user_id: u64,
    request_type: Type
}

fn main() -> Result<(), Error> {
    // let request = Request {
    //     side: Side::Buy,
    //     price: 1,
    //     size: 1,
    //     user_id: 1,
    //     request_type: Type::Limit
    // };
    // println!("{}", serde_json::to_string(&request)?);
    let data = include_str!("../requests.json");
    let requests: Vec<Request> = serde_json::from_str(data)?;
    println!("{}", serde_json::to_string(&requests)?);
        // testing whether correct order is maintained when adding new request
    let mut order_book = OrderBook::default();
    for request in requests {
        order_book.match_request(request);
    }
    Ok(())
}

#[derive(Default, Debug)]
struct OrderBook {
    buyers: Vec<Request>,
    sellers: Vec<Request>,
}

impl OrderBook {
    fn insert_limit_request(&mut self, request: Request) {
        match request.side {
            Side::Buy => {
                let search_result = self.buyers.binary_search_by(|probe| probe.price.cmp(&request.price));
                match search_result {
                    Err(i) => self.buyers.insert(i, request),
                    Ok(i) => {
                        let mut index = i + 1;
                        while index < self.buyers.len() && self.buyers[index].price == request.price {
                            index += 1;
                        }
                        self.buyers.insert(index, request);
                    }
                }
            },
            Side::Sell => {
                let search_result = self.sellers.binary_search_by(|probe| probe.price.cmp(&request.price).reverse());
                match search_result {
                    Err(i) => self.sellers.insert(i, request),
                    Ok(i) => {
                        let mut index = i + 1;
                        while index < self.buyers.len() && self.sellers[index].price == request.price {
                            index += 1;
                        }
                        self.sellers.insert(index, request);
                    }
                }
            },
        }
    }

    pub fn match_request(&mut self, request: Request) {
        let mut left = request.size;
        if left > 0 {
            match request.request_type {
                Type::Limit => {
                    self.insert_limit_request(request)
                }
                _ => {}
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adding_buy_limit_to_empty_book() {
        let mut book = OrderBook::default();
        let limit_request = Request {
            side: Side::Buy,
            price: 1,
            size: 1,
            user_id: 1,
            request_type: Type::Limit
        };
        book.match_request(limit_request);
    }

    #[test]
    fn test_adding_buy_limit_to_non_empty_book() {
        let mut book = OrderBook::default();
        let requests =
            (1..=10).flat_map(|i| {
                (1..=2).map(move |_| {
                    Request {
                        side: Side::Buy,
                        price: i,
                        size: 1,
                        user_id: i,
                        request_type: Type::Limit
                    }
                })
            });
        for request in requests {
            book.match_request(request);
        }
        for i in (0..=9){
            for j in (0..=1) {
                assert_eq!(book.buyers[i * 2 + j].price, (i + 1) as u64);
            }
        };
        let request =
            Request {
                side: Side::Buy,
                price: 11,
                size: 1,
                user_id: 24,
                request_type: Type::Limit
            };
        book.match_request(request);
        assert_eq!(book.buyers.last().unwrap().price, 11);
        // testing whether correct order is maintained when adding new request
        let request =
            Request {
                side: Side::Buy,
                price: 2,
                size: 1,
                user_id: 24,
                request_type: Type::Limit
            };
        book.match_request(request);
        assert_eq!(book.buyers[4].price, 2);
        assert_eq!(book.buyers[4].user_id, 24);
    }

    #[test]
    fn test_adding_sell_limit_to_empty_book() {
        let mut book = OrderBook::default();
        let limit_request = Request {
            side: Side::Sell,
            price: 1,
            size: 1,
            user_id: 1,
            request_type: Type::Limit
        };
        book.match_request(limit_request);
    }

    #[test]
    fn test_adding_sell_limit_to_non_empty_book() {
        let mut book = OrderBook::default();
        let requests =
            (1..=10).flat_map(|i| {
                (1..=2).map(move |_| {
                    Request {
                        side: Side::Sell,
                        price: i,
                        size: 1,
                        user_id: i,
                        request_type: Type::Limit
                    }
                })
            });
        for request in requests {
            book.match_request(request);
        }
        for i in (0..=9){
            for j in (0..=1) {
                assert_eq!(book.sellers[i * 2 + j].price, (10 - i) as u64);
            }
        };
        let request =
            Request {
                side: Side::Sell,
                price: 11,
                size: 1,
                user_id: 24,
                request_type: Type::Limit
            };
        book.match_request(request);
        assert_eq!(book.sellers.first().unwrap().price, 11);
        // testing whether correct order is maintained when adding new request
        let request =
            Request {
                side: Side::Sell,
                price: 10,
                size: 1,
                user_id: 24,
                request_type: Type::Limit
            };
        book.match_request(request);
        assert_eq!(book.sellers[3].price, 10);
        assert_eq!(book.sellers[3].user_id, 24);
    }
}
