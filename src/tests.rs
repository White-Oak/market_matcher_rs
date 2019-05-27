use crate::matcher::*;

#[test]
fn test_adding_buy_limit_to_empty_book() {
    let mut book = OrderBook::default();
    let limit_request = Request {
        side: Side::Buy,
        price: 1,
        size: 1,
        user_id: 1,
        request_type: Type::Limit,
    };
    let matching_result = book.match_request(&limit_request);
    let expected = MatchingResult {
        market_actions: vec![],
        request_actions: vec![RequestAction::AddedToBook],
    };
    assert_eq!(matching_result, expected);
    assert_eq!(book.buyers.len(), 1);
    assert_eq!(book.sellers.len(), 0);
    assert_eq!(book.buyers[0], limit_request);
}

#[test]
fn test_adding_sell_limit_to_empty_book() {
    let mut book = OrderBook::default();
    let limit_request = Request {
        side: Side::Sell,
        price: 1,
        size: 1,
        user_id: 1,
        request_type: Type::Limit,
    };
    let matching_result = book.match_request(&limit_request);
    let expected = MatchingResult {
        market_actions: vec![],
        request_actions: vec![RequestAction::AddedToBook],
    };
    assert_eq!(matching_result, expected);
    assert_eq!(book.buyers.len(), 0);
    assert_eq!(book.sellers.len(), 1);
    assert_eq!(book.sellers[0], limit_request);
}

#[test]
fn test_adding_buy_limit_to_non_empty_book() {
    let mut book = OrderBook::default();
    // generating 20 requests with prices from 1 to 10
    let requests = (1..=10).flat_map(|i| {
        (1..=2).map(move |_| Request {
            side: Side::Buy,
            price: i,
            size: 1,
            user_id: i,
            request_type: Type::Limit,
        })
    });
    for request in requests {
        book.match_request(&request);
    }
    // asserting that buy book is sorted from the highest to the lowest point
    for i in 0..=9 {
        for j in 0..=1 {
            assert_eq!(book.buyers[i * 2 + j].price, (10 - i) as u64);
        }
    }
    let request = Request {
        side: Side::Buy,
        price: 11,
        size: 1,
        user_id: 24,
        request_type: Type::Limit,
    };
    let matching_result = book.match_request(&request);
    let expected = MatchingResult {
        market_actions: vec![],
        request_actions: vec![RequestAction::AddedToBook],
    };
    assert_eq!(matching_result, expected);
    assert_eq!(book.buyers[0], request);
    // testing whether correct order (by time, the first is earliest) is maintained when adding new request
    let request = Request {
        side: Side::Buy,
        price: 10,
        size: 1,
        user_id: 24,
        request_type: Type::Limit,
    };
    let matching_result = book.match_request(&request);
    assert_eq!(matching_result, expected);
    assert_eq!(book.buyers[3], request);
}

#[test]
fn test_adding_sell_limit_to_non_empty_book() {
    let mut book = OrderBook::default();
    // generating 20 requests with prices from 2 to 10
    let requests = (2..=10).flat_map(|i| {
        (1..=2).map(move |_| Request {
            side: Side::Sell,
            price: i,
            size: 1,
            user_id: i,
            request_type: Type::Limit,
        })
    });
    for request in requests {
        book.match_request(&request);
    }
    // asserting that sell book is sorted from the lowest to the highest price
    for i in 0..9 {
        for j in 0..2 {
            assert_eq!(book.sellers[i * 2 + j].price, (i + 2) as u64);
        }
    }
    // testing that new low enough item will be added at the beginning
    let request = Request {
        side: Side::Sell,
        price: 1,
        size: 1,
        user_id: 24,
        request_type: Type::Limit,
    };
    let matching_result = book.match_request(&request);
    let expected = MatchingResult {
        market_actions: vec![],
        request_actions: vec![RequestAction::AddedToBook],
    };
    assert_eq!(matching_result, expected);
    assert_eq!(book.sellers[0], request);
    // testing whether correct order (FIFO by time, the first is earliest)
    // is maintained when adding new request
    let request = Request {
        side: Side::Sell,
        price: 2,
        size: 1,
        user_id: 24,
        request_type: Type::Limit,
    };
    let matching_result = book.match_request(&request);
    assert_eq!(matching_result, expected);
    assert_eq!(book.sellers[3], request);
}

#[test]
fn test_simple_limit_matching_one_to_one() {
    let mut book = OrderBook::default();
    let mut limit_request = Request {
        side: Side::Buy,
        price: 1,
        size: 1,
        user_id: 1,
        request_type: Type::Limit,
    };
    book.match_request(&limit_request.clone());
    limit_request.side = Side::Sell;
    // should not sell to the same user
    let matching_result = book.match_request(&limit_request);
    let expected = MatchingResult {
        market_actions: vec![],
        request_actions: vec![RequestAction::AddedToBook],
    };
    assert_eq!(matching_result, expected);
    assert_eq!(book.sellers.len(), 1);
    assert_eq!(book.buyers.len(), 1);
    limit_request.user_id = 2;
    // should sell to other user
    let matching_result = book.match_request(&limit_request);
    let expected = MatchingResult {
        market_actions: vec![MarketAction {
            size: 1,
            price: 1,
            seller_user_id: 2,
            buyer_user_id: 1,
        }],
        request_actions: vec![RequestAction::Filled],
    };
    assert_eq!(matching_result, expected);
    assert_eq!(book.sellers.len(), 1);
    assert_eq!(book.buyers.len(), 0);
    limit_request.side = Side::Buy;
    // should buy from other user
    let matching_result = book.match_request(&limit_request);
    let expected = MatchingResult {
        market_actions: vec![MarketAction {
            size: 1,
            price: 1,
            seller_user_id: 1,
            buyer_user_id: 2,
        }],
        request_actions: vec![RequestAction::Filled],
    };
    assert_eq!(matching_result, expected);
    assert_eq!(book.sellers.len(), 0);
    assert_eq!(book.buyers.len(), 0);
}

#[test]
fn test_correct_logic_of_buying_matching() {
    let mut book = OrderBook::default();
    for i in 1..=5 {
        let limit_request = Request {
            side: Side::Sell,
            price: i,
            size: 1,
            user_id: 1,
            request_type: Type::Limit,
        };
        book.match_request(&limit_request);
    }
    for i in 5..10 {
        let limit_request = Request {
            side: Side::Buy,
            price: i,
            size: 1,
            user_id: 2,
            request_type: Type::Limit,
        };
        let matching_result = book.match_request(&limit_request);
        let expected = MatchingResult {
            market_actions: vec![MarketAction {
                size: 1,
                price: i - 4,
                seller_user_id: 1,
                buyer_user_id: 2,
            }],
            request_actions: vec![RequestAction::Filled],
        };
        assert_eq!(book.sellers.len(), (10 - i - 1) as usize);
        assert_eq!(book.buyers.len(), 0);
        assert_eq!(matching_result, expected);
    }
}

#[test]
fn test_correct_logic_of_selling_matching() {
    let mut book = OrderBook::default();
    for i in 6..=10 {
        let limit_request = Request {
            side: Side::Buy,
            price: i,
            size: 1,
            user_id: 1,
            request_type: Type::Limit,
        };
        book.match_request(&limit_request);
    }
    for i in 1..=5 {
        let limit_request = Request {
            side: Side::Sell,
            price: i,
            size: 1,
            user_id: 2,
            request_type: Type::Limit,
        };
        let matching_result = book.match_request(&limit_request);
        let expected = MatchingResult {
            market_actions: vec![MarketAction {
                size: 1,
                price: 10 - i + 1,
                seller_user_id: 2,
                buyer_user_id: 1,
            }],
            request_actions: vec![RequestAction::Filled],
        };
        assert_eq!(book.buyers.len(), (5 - i) as usize);
        assert_eq!(book.sellers.len(), 0);
        assert_eq!(matching_result, expected);
    }
}

#[test]
fn test_limit_matching_with_leftovers() {
    let mut book = OrderBook::default();
    let mut limit_request = Request {
        side: Side::Buy,
        price: 1,
        size: 1,
        user_id: 1,
        request_type: Type::Limit,
    };
    book.match_request(&limit_request);
    limit_request.side = Side::Sell;
    limit_request.user_id = 2;
    limit_request.size = 5;
    // lets try selling 5 pieces to an order with 1 piece
    book.match_request(&limit_request);
    assert_eq!(book.sellers.len(), 1);
    assert_eq!(book.buyers.len(), 0);
    limit_request.size = 4;
    // lets check if a leftover is saved properly
    assert_eq!(book.sellers[0], limit_request);
}

#[test]
fn test_spread_limit_matching() {
    let mut book = OrderBook::default();
    for i in 1..=100 {
        let limit_request = Request {
            side: Side::Buy,
            price: i,
            size: 2,
            user_id: i,
            request_type: Type::Limit,
        };
        book.match_request(&limit_request);
    }
    for i in 101..=200 {
        let limit_request = Request {
            side: Side::Sell,
            price: i,
            size: 2,
            user_id: i,
            request_type: Type::Limit,
        };
        book.match_request(&limit_request);
    }
    let mut limit_request = Request {
        side: Side::Sell,
        price: 1,
        size: 300,
        user_id: 1000,
        request_type: Type::Limit,
    };
    // let's cover all of the buy offers and leave 100 in a book
    book.match_request(&limit_request);
    assert_eq!(book.sellers.len(), 101);
    assert_eq!(book.buyers.len(), 0);
    limit_request.side = Side::Buy;
    limit_request.price = 1000;
    // let's cover all of the sell offers (except previous one cause it's from the same user)
    // and leave 100 in book
    book.match_request(&limit_request);
    assert_eq!(book.sellers.len(), 1);
    assert_eq!(book.buyers.len(), 1);
    assert_eq!(book.buyers[0].size, 100);
    assert_eq!(book.sellers[0].size, 100);
}

#[test]
fn test_simple_fill_or_kill_selling_one_to_one() {
    let mut book = OrderBook::default();
    let limit_request = Request {
        side: Side::Buy,
        price: 1,
        size: 2,
        user_id: 1,
        request_type: Type::Limit,
    };
    book.match_request(&limit_request);
    let mut fk_request = Request {
        side: Side::Sell,
        price: 1,
        size: 2,
        user_id: 1,
        request_type: Type::FillOrKill,
    };
    // same user_id shouldn't sell to the book
    book.match_request(&fk_request);
    assert_eq!(book.sellers.len(), 0);
    assert_eq!(book.buyers.len(), 1);
    fk_request.user_id = 2;
    fk_request.size = 3;
    // unfilled incoming request should pass by
    book.match_request(&fk_request);
    assert_eq!(book.sellers.len(), 0);
    assert_eq!(book.buyers.len(), 1);
    fk_request.size = 2;
    // filled incoming request should be matched
    book.match_request(&fk_request);
    assert_eq!(book.sellers.len(), 0);
    assert_eq!(book.buyers.len(), 0);
}

#[test]
fn test_simple_fill_or_kill_bying_one_to_one() {
    let mut book = OrderBook::default();
    let limit_request = Request {
        side: Side::Sell,
        price: 1,
        size: 2,
        user_id: 1,
        request_type: Type::Limit,
    };
    book.match_request(&limit_request);
    let mut fk_request = Request {
        side: Side::Buy,
        price: 1,
        size: 2,
        user_id: 1,
        request_type: Type::FillOrKill,
    };
    // same user_id shouldn't sell to the book
    book.match_request(&fk_request);
    assert_eq!(book.sellers.len(), 1);
    assert_eq!(book.buyers.len(), 0);
    fk_request.user_id = 2;
    fk_request.size = 3;
    // unfilled incoming request should pass by
    book.match_request(&fk_request);
    assert_eq!(book.sellers.len(), 1);
    assert_eq!(book.buyers.len(), 0);
    fk_request.size = 2;
    // filled incoming request should be matched
    book.match_request(&fk_request);
    assert_eq!(book.sellers.len(), 0);
    assert_eq!(book.buyers.len(), 0);
}

#[test]
fn test_spread_fill_or_kill_selling() {
    let mut book = OrderBook::default();
    let limit_request = Request {
        side: Side::Buy,
        price: 1,
        size: 1,
        user_id: 1,
        request_type: Type::Limit,
    };
    for _ in 0..100 {
        book.match_request(&limit_request);
    }
    let mut fk_request = Request {
        side: Side::Sell,
        price: 1,
        size: 101,
        user_id: 2,
        request_type: Type::FillOrKill,
    };
    // shouldn't sell when is not satisfied
    book.match_request(&fk_request);
    assert_eq!(book.sellers.len(), 0);
    assert_eq!(book.buyers.len(), 100);
    fk_request.size = 100;
    // filled incoming request should be matched
    book.match_request(&fk_request);
    assert_eq!(book.sellers.len(), 0);
    assert_eq!(book.buyers.len(), 0);
}

#[test]
fn test_spread_fill_or_kill_buying() {
    let mut book = OrderBook::default();
    let limit_request = Request {
        side: Side::Sell,
        price: 1,
        size: 1,
        user_id: 1,
        request_type: Type::Limit,
    };
    for _ in 0..100 {
        book.match_request(&limit_request);
    }
    let mut fk_request = Request {
        side: Side::Buy,
        price: 1,
        size: 101,
        user_id: 2,
        request_type: Type::FillOrKill,
    };
    // shouldn't sell when is not satisfied
    book.match_request(&fk_request);
    assert_eq!(book.sellers.len(), 100);
    assert_eq!(book.buyers.len(), 0);
    fk_request.size = 100;
    // filled incoming request should be matched
    book.match_request(&fk_request);
    assert_eq!(book.sellers.len(), 0);
    assert_eq!(book.buyers.len(), 0);
}

#[test]
fn test_simple_immediate_or_cancel_selling_one_to_one() {
    let mut book = OrderBook::default();
    let limit_request = Request {
        side: Side::Buy,
        price: 1,
        size: 3,
        user_id: 1,
        request_type: Type::Limit,
    };
    book.match_request(&limit_request);
    let mut ic_request = Request {
        side: Side::Sell,
        price: 1,
        size: 1,
        user_id: 1,
        request_type: Type::ImmediateOrCancel,
    };
    // same user_id shouldn't sell to the book
    book.match_request(&ic_request);
    assert_eq!(book.sellers.len(), 0);
    assert_eq!(book.buyers.len(), 1);
    assert_eq!(book.buyers[0].size, 3);
    ic_request.user_id = 2;
    // unfilled incoming request should pass by
    book.match_request(&ic_request);
    assert_eq!(book.sellers.len(), 0);
    assert_eq!(book.buyers.len(), 1);
    assert_eq!(book.buyers[0].size, 2);
    ic_request.size = 2;
    // filled incoming request should be matched
    book.match_request(&ic_request);
    assert_eq!(book.sellers.len(), 0);
    assert_eq!(book.buyers.len(), 0);
}

#[test]
fn test_simple_immediate_or_cancel_buying_one_to_one() {
    let mut book = OrderBook::default();
    let limit_request = Request {
        side: Side::Sell,
        price: 1,
        size: 3,
        user_id: 1,
        request_type: Type::Limit,
    };
    book.match_request(&limit_request);
    let mut ic_request = Request {
        side: Side::Buy,
        price: 1,
        size: 1,
        user_id: 1,
        request_type: Type::ImmediateOrCancel,
    };
    // same user_id shouldn't buy from the book
    book.match_request(&ic_request);
    assert_eq!(book.sellers.len(), 1);
    assert_eq!(book.buyers.len(), 0);
    assert_eq!(book.sellers[0].size, 3);
    ic_request.user_id = 2;
    // unfilled incoming request should pass by
    book.match_request(&ic_request);
    assert_eq!(book.sellers.len(), 1);
    assert_eq!(book.buyers.len(), 0);
    assert_eq!(book.sellers[0].size, 2);
    ic_request.size = 2;
    // filled incoming request should be matched
    book.match_request(&ic_request);
    assert_eq!(book.sellers.len(), 0);
    assert_eq!(book.buyers.len(), 0);
}
