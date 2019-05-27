use serde::{Deserialize, Serialize};
use std::cmp::{self, Ordering};

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
    pub buyers: Buyers,
    pub sellers: Sellers,
}

#[derive(Default, Debug, Clone)]
pub struct Buyers {
    pub buyers: Vec<Request>
}

#[derive(Default, Debug, Clone)]
pub struct Sellers {
    pub sellers: Vec<Request>
}

trait OrderBookPart {
    fn target_vec(&mut self) -> &mut Vec<Request>;
    fn compare_prices_for_inserting(a: &Request, b: &Request) -> Ordering;
    fn can_match_prices(a: &Request, b: &Request) -> bool;
    fn build_market_action_to_apply(a: &Request, b: &Request,
                                    max_allowed: u64, current_index: usize) -> MarketActionToApply;

    fn insert_limit_request_common(&mut self, request: Request)
    {
        let search_result = self.target_vec()
            .binary_search_by(|probe| Self::compare_prices_for_inserting(probe, &request));
        match search_result {
            Err(i) => self.target_vec().insert(i, request),
            Ok(i) => {
                let mut index = i + 1;
                while index < self.target_vec().len()
                    && self.target_vec()[index].price == request.price {
                    index += 1;
                }
                self.target_vec().insert(index, request);
            }
        }
    }

    fn build_market_actions_and_apply_requests(opposite_vec: &mut Vec<Request>,
                                               request: &Request)
        -> (u64, Vec<MarketActionToApply>) {
        let mut left = request.size;
        let mut market_actions = Vec::new();
        let mut current_index = 0;
        while left > 0 {
            if let Some(mut from_book) = opposite_vec.get_mut(current_index) {
                if from_book.user_id == request.user_id {
                    current_index += 1;
                    continue
                }
                if !Self::can_match_prices(from_book, request) {
                    // we can sell only higher or equal to an order
                    break
                }
                let max_allowed = cmp::min(from_book.size, left);
                left -= max_allowed;
                let market_action = Self::build_market_action_to_apply(from_book, request,
                                                                       max_allowed, current_index);
                market_actions.push(market_action);
                match request.request_type {
                    Type::Limit | Type::ImmediateOrCancel => {
                        if from_book.size == max_allowed {
                            // if an order was fully satisfied we remove it from the book
                            opposite_vec.remove(current_index);
                        } else {
                            // else change its amount
                            from_book.size -= max_allowed
                        }
                    },
                    Type::FillOrKill => {
                        // if an order was fully satisfied we need to move to the next item
                        // in list
                        if from_book.size == max_allowed {
                            current_index += 1;
                        }
                    }
                }
            } else {
                // if there are no buyers left, we cannot sell anymore
                break
            }
        }
        (left, market_actions)
    }

    fn apply_market_actions(opposite_vec: &mut Vec<Request>, market_actions: &Vec<MarketActionToApply>) {
        // we are going in reverse so the first action has the biggest index,
        // and the last one has the smallest.
        // this way, we can delete satisfied orders from book, because every
        // following action has a smaller index in the book than the current one
        for market_action in market_actions.iter().rev() {
            let mut changed_request = &mut opposite_vec[market_action.index_in_book];
            // if sizes are equal, an order in book was satisfied fully
            // so we can delete it
            if changed_request.size == market_action.size {
                opposite_vec.remove(market_action.index_in_book);
                continue
            }
            // else, change the size of the order
            changed_request.size -= market_action.size;
        }
    }

    fn match_request(&mut self, opposite_vec: &mut Vec<Request>, request: &Request) -> MatchingResult {
        // first, try to fill incoming request from the book
        // we also build a vec of MarketActionToApply for two reasons:
        // 1) we can output exactly what has been sold
        // 2) for FillOrKill we will need to apply those actions if incoming request was fully
        //    satisfied
        let (left, market_actions) = Self::build_market_actions_and_apply_requests(opposite_vec, request);

        let mut request_actions = Vec::new();
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
                    self.insert_limit_request_common(leftover_request);
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
                    // when an incoming request buys, changes are applied
                    Self::apply_market_actions(opposite_vec, &market_actions);
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

impl OrderBookPart for Buyers {
    fn target_vec(&mut self) -> &mut Vec<Request>{
        &mut self.buyers
    }
    // comparing in reverse so buyers is sorted in descending order
    fn compare_prices_for_inserting(a: &Request, b: &Request) -> Ordering{
        a.price.cmp(&b.price).reverse()
    }

    fn can_match_prices(seller: &Request, request: &Request) -> bool {
        // we can buy only lower or equal to an order
        seller.price <= request.price
    }

    fn build_market_action_to_apply(seller: &Request, request: &Request,
                                    max_allowed: u64, current_index: usize) -> MarketActionToApply {
        MarketActionToApply {
            size: max_allowed,
            price: seller.price,
            seller_user_id: seller.user_id,
            buyer_user_id: request.user_id,
            index_in_book: current_index
        }
    }
}

impl OrderBookPart for Sellers {
    fn target_vec(&mut self) -> &mut Vec<Request>{
        &mut self.sellers
    }
    // comparing so sellers is sorted in ascending order
    fn compare_prices_for_inserting(a: &Request, b: &Request) -> Ordering{
        a.price.cmp(&b.price)
    }

    fn can_match_prices(buyer: &Request, request: &Request) -> bool {
        // we can sell only higher or equal to an order
        buyer.price >= request.price
    }

    fn build_market_action_to_apply(buyer: &Request, request: &Request,
                                    max_allowed: u64, current_index: usize) -> MarketActionToApply {
        MarketActionToApply {
            size: max_allowed,
            price: buyer.price,
            seller_user_id: request.user_id,
            buyer_user_id: buyer.user_id,
            index_in_book: current_index
        }
    }
}


impl OrderBook {

    pub fn match_request<'a>(&mut self, request: &'a Request) -> MatchingResult {
        match request.side {
            Side::Buy => self.buyers.match_request(self.sellers.target_vec(), request),
            Side::Sell => self.sellers.match_request(self.buyers.target_vec(), request)
        }
    }

    pub fn buyers(&self) -> &Vec<Request> {
        &self.buyers.buyers
    }

    pub fn sellers(&self) -> &Vec<Request> {
        &self.sellers.sellers
    }
}

