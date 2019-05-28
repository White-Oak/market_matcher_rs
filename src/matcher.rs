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

struct MarketActionToApply {
    size: u64,
    price: u64,
    seller_user_id: u64,
    buyer_user_id: u64,
    index_in_book: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MarketAction {
    pub size: u64,
    pub price: u64,
    pub seller_user_id: u64,
    pub buyer_user_id: u64,
}

impl From<MarketActionToApply> for MarketAction {
    fn from(mata: MarketActionToApply) -> Self {
        MarketAction {
            size: mata.size,
            price: mata.price,
            seller_user_id: mata.seller_user_id,
            buyer_user_id: mata.buyer_user_id,
        }
    }
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
    pub buyers: Vec<Request>,
    pub sellers: Vec<Request>,
}

impl OrderBook {
    fn insert_limit_request(&mut self, request: Request) {
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

    pub fn match_request<'a>(&mut self, request: &'a Request) -> MatchingResult {
        let mut left = request.size;
        let mut market_actions = Vec::new();
        let mut request_actions = Vec::new();
        let mut ranges = Vec::new();
        let opposite_vec = match request.side {
            Side::Buy => &mut self.sellers,
            Side::Sell => &mut self.buyers
        };

        let mut previous_left_border = 0;
        let mut current_index = 0;
        while left > 0 {
            // println!("left border {}, curr index {}", previous_left_border, current_index);
            if let Some(passive_request) = opposite_vec.get(current_index) {
                if passive_request.user_id == request.user_id {
                    if previous_left_border != current_index {
                        ranges.push(previous_left_border..current_index);
                    }
                    current_index += 1;
                    previous_left_border = current_index;
                    continue;
                }
                let max_allowed = cmp::min(passive_request.size, left);
                let market_action =
                    match request.side {
                        Side::Sell => {
                            if passive_request.price < request.price {
                                // we can sell only higher or equal to an order
                                break;
                            }
                            MarketActionToApply {
                                size: max_allowed,
                                price: passive_request.price,
                                seller_user_id: request.user_id,
                                buyer_user_id: passive_request.user_id,
                                index_in_book: current_index,
                            }
                        },
                        Side::Buy => {
                            if passive_request.price > request.price {
                                // we can buy only lower or equal to an order
                                break;
                            }
                            MarketActionToApply {
                                size: max_allowed,
                                price: passive_request.price,
                                seller_user_id: passive_request.user_id,
                                buyer_user_id: request.user_id,
                                index_in_book: current_index,
                            }
                        }
                    };
                left -= max_allowed;
                market_actions.push(market_action);
                current_index += 1;
                if max_allowed != passive_request.size {
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
            for market_action in market_actions.iter() {
                let mark = market_action.index_in_book;
                let mut changed_request =
                    &mut opposite_vec[mark];
                // if sizes are not equal we need to change the order in book
                if changed_request.size != market_action.size {
                    changed_request.size -= market_action.size;
                    continue;
                }
            }
            for range in ranges.into_iter().rev() {
                opposite_vec.drain(range);
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
            market_actions: market_actions.into_iter().map(|i| i.into()).collect(),
            request_actions,
        }
    }
}
