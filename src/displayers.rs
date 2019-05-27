use std::fmt::*;
use std::string::ToString;

use crate::matcher::*;

impl Display for MarketAction {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "User #{} sold {} pieces at price point '{}' to user #{}",
            self.seller_user_id, self.size, self.price, self.buyer_user_id
        )
    }
}

impl Display for RequestAction {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let res_str = match self {
            RequestAction::Filled => "satisfied",
            RequestAction::FilledPartially => "satisfied partially",
            RequestAction::Cancelled => "cancelled",
            RequestAction::AddedToBook => "added to the market",
        };
        write!(f, "{}", res_str)
    }
}

impl Display for MatchingResult {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut message = String::new();
        let request_actions = self
            .request_actions
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        message += &format!("Request was {}", request_actions);

        if !self.market_actions.is_empty() {
            let market_actions = self
                .market_actions
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n");
            message += &format!(
                " and the following actions were performed on the market:\n{}",
                market_actions
            );
        }
        write!(f, "{}", message)
    }
}

impl Display for Request {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let side_str = match self.side {
            Side::Sell => "sell",
            Side::Buy => "buy",
        };
        let type_str = match self.request_type {
            Type::Limit => "Limit",
            Type::ImmediateOrCancel => "Immediate or cancel",
            Type::FillOrKill => "Fill or kill",
        };
        write!(
            f,
            "Incoming {} request from user #{} to {} {} pieces at price point '{}'",
            type_str, self.user_id, side_str, self.size, self.price
        )
    }
}
