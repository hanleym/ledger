use crate::event_store::{Event, EventType, Projector};
use crate::{ClientID, Money, TransactionID};
use anyhow::{Context, Result, anyhow};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default, Clone)]
struct Account {
    available: Money,
    held: Money,
    disputed: BTreeSet<TransactionID>,
    locked: bool,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Output {
    client: ClientID,
    available: Money,
    held: Money,
    total: Money,
    locked: bool,
}

#[cfg(test)]
impl Output {
    pub fn new<M: Into<Money>, H: Into<Money>>(
        client: ClientID,
        available: M,
        held: H,
        locked: bool,
    ) -> Self {
        let available = available.into();
        let held = held.into();
        Self {
            client,
            available,
            held,
            total: available + held,
            locked,
        }
    }
}

impl From<(ClientID, Account)> for Output {
    fn from((client, account): (ClientID, Account)) -> Self {
        Self {
            client,
            available: account.available,
            held: account.held,
            total: account.available + account.held,
            locked: account.locked,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BalanceSheet {
    accounts: BTreeMap<ClientID, Account>,
}

impl BalanceSheet {
    pub fn new() -> Self {
        Self {
            accounts: BTreeMap::new(),
        }
    }

    pub fn write_csv<W: std::io::Write>(self, writer: W) -> Result<()> {
        let mut writer = csv::Writer::from_writer(writer);
        for account in self.accounts.into_iter().map(Output::from) {
            writer
                .serialize(account)
                .context("Failed to write to CSV")?;
        }
        writer.flush().context("Failed to flush CSV")?;
        Ok(())
    }
}

impl Projector for BalanceSheet {
    fn project(&mut self, event: &Event) -> Result<()> {
        let account = self.accounts.entry(event.client).or_default();
        if account.locked {
            return Err(anyhow!("Account locked: {}", event.client));
        }
        let amount = event.amount.unwrap_or_default();
        match event._type {
            EventType::Deposit => {
                account.available += amount;
            }
            EventType::Withdrawal => {
                if account.available >= amount {
                    account.available -= amount;
                } else {
                    return Err(anyhow!("Insufficient funds for withdrawal: {}", amount));
                }
            }
            EventType::Dispute => {
                if account.disputed.contains(&event.tx) {
                    return Err(anyhow!("Transaction already disputed: {}", event.tx));
                }
                account.disputed.insert(event.tx);
                account.available -= amount;
                account.held += amount;
            }
            EventType::Resolve => {
                if !account.disputed.contains(&event.tx) {
                    return Err(anyhow!("Transaction not disputed: {}", event.tx));
                }
                account.disputed.remove(&event.tx);
                account.available += amount;
                account.held -= amount;
            }
            EventType::Chargeback => {
                if !account.disputed.contains(&event.tx) {
                    return Err(anyhow!("Transaction not disputed: {}", event.tx));
                }
                account.disputed.remove(&event.tx);
                account.held -= amount;
                account.locked = true;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
impl BalanceSheet {
    pub fn into_output(self) -> Vec<Output> {
        self.accounts.into_iter().map(Output::from).collect()
    }
}
