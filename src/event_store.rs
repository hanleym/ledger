use crate::event_store::EventType::*;
use crate::{ClientID, Money, TransactionID};
use std::collections::BTreeMap;

use anyhow::{Result, anyhow};
use log::warn;
#[allow(unused_imports)]
use rust_decimal::prelude::Zero;
use serde::Deserialize;

pub trait Projector {
    fn project(&mut self, event: &Event) -> Result<()>;
}

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

#[cfg(test)]
impl Event {
    pub fn deposit<A: Into<Money>>(client: ClientID, tx: TransactionID, amount: A) -> Self {
        Self {
            _type: Deposit,
            client,
            tx,
            amount: Some(amount.into()),
        }
    }

    pub fn withdraw<A: Into<Money>>(client: ClientID, tx: TransactionID, amount: A) -> Self {
        Self {
            _type: Withdrawal,
            client,
            tx,
            amount: Some(amount.into()),
        }
    }

    pub fn dispute(client: ClientID, tx: TransactionID) -> Self {
        Self {
            _type: Dispute,
            client,
            tx,
            amount: None,
        }
    }

    pub fn resolve(client: ClientID, tx: TransactionID) -> Self {
        Self {
            _type: Resolve,
            client,
            tx,
            amount: None,
        }
    }

    pub fn chargeback(client: ClientID, tx: TransactionID) -> Self {
        Self {
            _type: Chargeback,
            client,
            tx,
            amount: None,
        }
    }
}

pub struct EventStore {
    events: Vec<Event>,
    index: BTreeMap<(ClientID, TransactionID), usize>,
}

impl EventStore {
    pub fn new() -> Self {
        Self {
            events: Default::default(),
            index: Default::default(),
        }
    }

    pub fn from_csv<R: std::io::Read>(reader: R) -> Result<Self> {
        let mut store = Self::new();
        store.load_csv(reader)?;
        Ok(store)
    }

    pub fn load_csv<R: std::io::Read>(&mut self, reader: R) -> Result<()> {
        let mut reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_reader(reader);
        for record in reader.deserialize() {
            if let Err(err) = self.save(record?) {
                warn!("{err}");
            }
        }
        Ok(())
    }

    fn enrich(&mut self, event: &mut Event) -> Result<()> {
        match event._type {
            Dispute | Resolve | Chargeback => {
                let index = self
                    .index
                    .get(&(event.client, event.tx))
                    .ok_or_else(|| anyhow!("Transaction not found; skipping."))?;
                let transaction = &mut self
                    .events
                    .get(*index)
                    .ok_or_else(|| anyhow!("Event not found; skipping."))?;
                event.amount = transaction.amount;
            }
            _ => {}
        }
        Ok(())
    }

    fn commit(&mut self, event: Event) {
        self.index
            .insert((event.client, event.tx), self.events.len());
        self.events.push(event);
    }

    pub fn save(&mut self, mut event: Event) -> Result<()> {
        self.enrich(&mut event)?;
        self.commit(event);
        Ok(())
    }

    pub fn project(&self, projectors: &mut [&mut dyn Projector]) -> Result<()> {
        for event in &self.events {
            for projector in projectors.iter_mut() {
                if let Err(err) = projector.project(event) {
                    warn!("{err}");
                }
            }
        }
        Ok(())
    }
}
