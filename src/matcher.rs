use serde::{Deserialize, Serialize};
use std::cmp;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Limit,
    FillOrKill,
    ImmediateOrCancel,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct Request {
    pub side: Side,
    pub price: u64,
    pub size: u64,
    pub user_id: u64,
    pub request_type: Type,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MarketAction {
    pub size: u64,
    pub price: u64,
    pub seller_user_id: u64,
    pub buyer_user_id: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RequestAction {
    Filled,
    FilledPartially,
    Cancelled,
    AddedToBook,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MatchingResult {
    pub market_actions: Vec<MarketAction>,
    pub request_actions: Vec<RequestAction>,
}

#[derive(Default, Debug, Clone)]
pub struct OrderBook {
    pub buyers: RequestQueue,
    pub sellers: RequestQueue,
}

#[derive(Default, Debug, Clone)]
pub struct RequestQueue {
    pub vec: Vec<Request>,
    pub start_from: usize
}

use std::ops::{Deref, DerefMut};

impl RequestQueue {
    fn flush_vec(&mut self) {
        self.vec.drain(0..self.start_from);
        self.start_from = 0;
    }
}

impl Deref for RequestQueue {
    type Target = Vec<Request>;

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl DerefMut for RequestQueue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}

impl OrderBook {
    pub fn flush_request_queues(&mut self) {
        self.sellers.flush_vec();
        self.buyers.flush_vec();
    }

    fn insert_limit_request(&mut self, request: Request) {
        self.flush_request_queues();
        match request.side {
            Side::Buy => {
                // the order for buyers is from the highest to the lowest
                let search_result = self
                    .buyers
                    .binary_search_by(|probe| probe.price.cmp(&request.price).reverse());
                match search_result {
                    Err(i) => self.buyers.insert(i, request),
                    Ok(i) => {
                        let mut index = i + 1;
                        while index < self.buyers.len() && self.buyers[index].price == request.price
                        {
                            index += 1;
                        }
                        self.buyers.insert(index, request);
                    }
                }
            }
            Side::Sell => {
                // the order for sellers is from the lowest to the highest
                let search_result = self
                    .sellers
                    .binary_search_by(|probe| probe.price.cmp(&request.price));
                match search_result {
                    Err(i) => self.sellers.insert(i, request),
                    Ok(i) => {
                        let mut index = i + 1;
                        while index < self.sellers.len()
                            && self.sellers[index].price == request.price
                        {
                            index += 1;
                        }
                        self.sellers.insert(index, request);
                    }
                }
            }
        }
    }

    pub fn match_request_quiet<'a>(&mut self, request: &'a Request) {
        let mut left = request.size;
        let mut ranges = Vec::with_capacity(10);
        let opposite_vec = match request.side {
            Side::Buy => &mut self.sellers,
            Side::Sell => &mut self.buyers,
        };

        let mut previous_left_border = 0;
        let mut current_index = opposite_vec.start_from;
        while left > 0 {
            if let Some(mut passive_request) = opposite_vec.get_mut(current_index) {
                if passive_request.user_id == request.user_id {
                    if previous_left_border != current_index {
                        ranges.push(previous_left_border..current_index);
                    }
                    current_index += 1;
                    previous_left_border = current_index;
                    continue;
                }
                let max_allowed = cmp::min(passive_request.size, left);
                match request.side {
                    Side::Sell => {
                        if passive_request.price < request.price {
                            // we can sell only higher or equal to an order
                            break;
                        }
                    }
                    Side::Buy => {
                        if passive_request.price > request.price {
                            // we can buy only lower or equal to an order
                            break;
                        }
                    }
                };
                left -= max_allowed;
                current_index += 1;
                // if we can sell or buy less than passive request size
                // it means there is no point in moving further down the book
                if max_allowed != passive_request.size {
                    // so we modify passive request
                    // (we can do it, because this situation means incoming request was fully satisfied)
                    passive_request.size -= max_allowed;
                    // we shouldn't remove the passive request, so we back off a bit
                    current_index -= 1;
                    break;
                }
            } else {
                // if there are no passive requests left, we cannot sell anymore
                break;
            }
        }
        if previous_left_border != current_index {
            ranges.push(previous_left_border..current_index);
        }

        let is_fk = request.request_type == Type::FillOrKill;
        if left == 0 || !is_fk {
            for range in ranges.into_iter().rev() {
                if range.contains(&0) {
                    opposite_vec.start_from = range.end
                } else {
                    opposite_vec.drain(range);
                }
            }
        }

        // if there are leftovers from incoming request, save them to the book
        if left > 0 && request.request_type == Type::Limit {
            let leftover_request = Request {
                request_type: request.request_type,
                side: request.side,
                size: left,
                price: request.price,
                user_id: request.user_id,
            };
            self.insert_limit_request(leftover_request);
        }
    }

    pub fn match_request<'a>(&mut self, request: &'a Request) -> MatchingResult {
        let mut left = request.size;
        let mut market_actions = Vec::new();
        let mut request_actions = Vec::with_capacity(20);
        let mut ranges = Vec::with_capacity(10);
        let opposite_vec = match request.side {
            Side::Buy => &mut self.sellers,
            Side::Sell => &mut self.buyers,
        };

        let mut previous_left_border = 0;
        let mut current_index = opposite_vec.start_from;
        while left > 0 {
            // println!("left border {}, curr index {}", previous_left_border, current_index);
            if let Some(mut passive_request) = opposite_vec.get_mut(current_index) {
                if passive_request.user_id == request.user_id {
                    if previous_left_border != current_index {
                        ranges.push(previous_left_border..current_index);
                    }
                    current_index += 1;
                    previous_left_border = current_index;
                    continue;
                }
                let max_allowed = cmp::min(passive_request.size, left);
                let market_action = match request.side {
                    Side::Sell => {
                        if passive_request.price < request.price {
                            // we can sell only higher or equal to an order
                            break;
                        }
                        MarketAction {
                            size: max_allowed,
                            price: passive_request.price,
                            seller_user_id: request.user_id,
                            buyer_user_id: passive_request.user_id,
                        }
                    }
                    Side::Buy => {
                        if passive_request.price > request.price {
                            // we can buy only lower or equal to an order
                            break;
                        }
                        MarketAction {
                            size: max_allowed,
                            price: passive_request.price,
                            seller_user_id: passive_request.user_id,
                            buyer_user_id: request.user_id,
                        }
                    }
                };
                left -= max_allowed;
                market_actions.push(market_action);
                current_index += 1;
                // if we can sell or buy less than passive request size
                // it means there is no point in moving further down the book
                if max_allowed != passive_request.size {
                    // so we modify passive request
                    // (we can do it, because this situation means incoming request was fully satisfied)
                    passive_request.size -= max_allowed;
                    // we shouldn't remove the passive request, so we back off a bit
                    current_index -= 1;
                    break;
                }
            } else {
                // if there are no passive requests left, we cannot sell anymore
                break;
            }
        }
        // println!("After left border {}, curr index {}", previous_left_border, current_index);
        if previous_left_border != current_index {
            ranges.push(previous_left_border..current_index);
        }
        // println!("{:?}", ranges);

        let is_fk = request.request_type == Type::FillOrKill;
        if left == 0 || !is_fk {
            for range in ranges.into_iter().rev() {
                if range.contains(&0) {
                    opposite_vec.start_from = range.end
                } else {
                    opposite_vec.drain(range);
                }
            }
        }

        // building result
        if left > 0 {
            // if there are leftovers from incoming request, save them to the book
            match request.request_type {
                Type::Limit => {
                    let leftover_request = Request {
                        request_type: request.request_type,
                        side: request.side,
                        size: left,
                        price: request.price,
                        user_id: request.user_id,
                    };
                    self.insert_limit_request(leftover_request);
                    if left != request.size {
                        request_actions.push(RequestAction::FilledPartially);
                    }
                    request_actions.push(RequestAction::AddedToBook);
                }
                Type::FillOrKill => request_actions.push(RequestAction::Cancelled),
                Type::ImmediateOrCancel => {
                    if left != request.size {
                        request_actions.push(RequestAction::FilledPartially);
                    }
                    request_actions.push(RequestAction::Cancelled);
                }
            }
        } else {
            request_actions.push(RequestAction::Filled);
        }
        MatchingResult {
            market_actions,
            request_actions,
        }
    }
}
