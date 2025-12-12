use crate::{ClientID, Money, TransactionID};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(PartialEq)]
pub enum EventType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize)]
pub struct Event {
    #[serde(rename = "type")]
    pub _type: EventType,
    pub client: ClientID,
    pub tx: TransactionID,
    pub amount: Option<Money>,
}
