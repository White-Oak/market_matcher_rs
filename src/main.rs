extern crate failure;
extern crate serde_json;
extern crate serde;

use failure::Error;

mod matcher;
#[cfg(test)]
mod tests;

use crate::matcher::*;

fn main() -> Result<(), Error> {
    // let data = include_str!("../requests.json");
    // let requests: Vec<Request> = serde_json::from_str(data)?;
    // println!("{}", serde_json::to_string(&requests)?);
    //     // testing whether correct order is maintained when adding new request
    // let mut order_book = OrderBook::default();
    // for request in requests {
    //     order_book.match_request(&request);
    // }
    Ok(())
}

