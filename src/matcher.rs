use serde::{Deserialize, Serialize};
use std::cmp;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Limit,
    FillOrKill,
    ImmediateOrCancel
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct Request {
    pub side: Side,
    pub price: u64,
    pub size: u64,
    pub user_id: u64,
    pub request_type: Type
}

struct MarketActionToApply {
    size: u64,
    price: u64,
    seller_user_id: u64,
    buyer_user_id: u64,
    index_in_book: usize
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
    pub request_actions: Vec<RequestAction>
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
                let search_result = self.buyers
                    .binary_search_by(|probe| probe.price.cmp(&request.price).reverse());
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
                // the order for sellers is from the lowest to the highest
                let search_result = self.sellers
                    .binary_search_by(|probe| probe.price.cmp(&request.price));
                match search_result {
                    Err(i) => self.sellers.insert(i, request),
                    Ok(i) => {
                        let mut index = i + 1;
                        while index < self.sellers.len() && self.sellers[index].price == request.price {
                            index += 1;
                        }
                        self.sellers.insert(index, request);
                    }
                }
            },
        }
    }


    pub fn match_request<'a>(&mut self, request: &'a Request) -> MatchingResult {
        let mut left = request.size;
        let mut market_actions = Vec::new();
        let mut request_actions = Vec::new();

        match request.side {
            Side::Sell => {
                let mut current_index = 0;
                while left > 0 {
                    if let Some(mut buyer) = self.buyers.get_mut(current_index) {
                        if buyer.user_id == request.user_id {
                            current_index += 1;
                            continue
                        }
                        if buyer.price < request.price {
                            // we can sell only higher or equal to an order
                            break
                        }
                        let max_allowed = cmp::min(buyer.size, left);
                        left -= max_allowed;
                        let market_action = MarketActionToApply {
                            size: max_allowed,
                            price: buyer.price,
                            seller_user_id: request.user_id,
                            buyer_user_id: buyer.user_id,
                            index_in_book: current_index
                        };
                        market_actions.push(market_action);
                        match request.request_type {
                            Type::Limit | Type::ImmediateOrCancel => {
                                if buyer.size == max_allowed {
                                    // if an order was fully satisfied we remove it from the book
                                    self.buyers.remove(current_index);
                                } else {
                                    // else change its amount
                                    buyer.size -= max_allowed
                                }
                            },
                            Type::FillOrKill => {
                                // if an order was fully satisfied we need to move to the next item
                                // in list
                                if buyer.size == max_allowed {
                                    current_index += 1;
                                }
                            }
                        }
                    } else {
                        // if there are no buyers left, we cannot sell anymore
                        break
                    }
                }
            },
            Side::Buy => {
                let mut current_index = 0;
                while left > 0 {
                    if let Some(seller) = self.sellers.get_mut(current_index) {
                        if seller.user_id == request.user_id {
                            current_index += 1;
                            continue
                        }
                        if seller.price > request.price {
                            // we can buy only lower or equal to an order
                            break
                        }
                        let max_allowed = cmp::min(seller.size, left);
                        left -= max_allowed;
                        let market_action = MarketActionToApply {
                            size: max_allowed,
                            price: seller.price,
                            seller_user_id: seller.user_id,
                            buyer_user_id: request.user_id,
                            index_in_book: current_index
                        };
                        market_actions.push(market_action);
                        match request.request_type {
                            Type::Limit | Type::ImmediateOrCancel => {
                                if seller.size == max_allowed {
                                    // if an order was fully satisfied we remove it from the book
                                    self.sellers.remove(current_index);
                                } else {
                                    // else change its amount
                                    seller.size -= max_allowed
                                }
                            },
                            Type::FillOrKill => {
                                // if an order was fully satisfied we need to move to the next item
                                // in list
                                if seller.size == max_allowed {
                                    current_index += 1;
                                }
                            }
                        }
                    } else {
                        // if there are no buyers left, we cannot sell anymore
                        break
                    }
                }
            },
        }
        if left > 0 {
            // if there are leftovers from incoming request, save them to the book
            match request.request_type {
                Type::Limit => {
                    let leftover_request = Request {
                        request_type: request.request_type,
                        side: request.side,
                        size: left,
                        price: request.price,
                        user_id: request.user_id
                    };
                    self.insert_limit_request(leftover_request);
                    if left != request.size {
                        request_actions.push(RequestAction::FilledPartially);
                    }
                    request_actions.push(RequestAction::AddedToBook);
                },
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
            // if fill or kill request was fully satisfied, apply the changes to the book
            match request.request_type {
                Type::FillOrKill => {
                    match request.side {
                        // when an incoming request buys, changes are applied to sellers
                        Side::Buy => {
                            // we are going in reverse so the first action has the biggest index,
                            // and the last one has the smallest.
                            // this way, we can delete satisfied orders from book, because every
                            // following action has a smaller index in the book than the current one
                            for market_action in market_actions.iter().rev() {
                                let mut changed_request = &mut self.sellers[market_action.index_in_book];
                                // if sizes are equal, an order in book was satisfied fully
                                // so we can delete it
                                if changed_request.size == market_action.size {
                                    self.sellers.remove(market_action.index_in_book);
                                    continue
                                }
                                // else, change the size of the order
                                changed_request.size -= market_action.size;
                            }
                        },
                        // when an incoming request sells, changes are applied to buyers
                        // Side::Sell => unimplemented!()
                        Side::Sell => {
                            // we are going in reverse so the first action has the biggest index,
                            // and the last one has the smallest.
                            // this way, we can delete satisfied orders from book, because every
                            // following action has a smaller index in the book than the current one
                            for market_action in market_actions.iter().rev() {
                                let mut changed_request = &mut self.buyers[market_action.index_in_book];
                                // if sizes are equal, an order in book was satisfied fully
                                // so we can delete it
                                if changed_request.size == market_action.size {
                                    self.buyers.remove(market_action.index_in_book);
                                    continue
                                }
                                // else, change the size of the order
                                changed_request.size -= market_action.size;
                            }
                        },
                    }
                },
                _ => {}
            }
        }
        MatchingResult {
            market_actions: market_actions.into_iter().map(|i| i.into()).collect(),
            request_actions
        }
    }
}

