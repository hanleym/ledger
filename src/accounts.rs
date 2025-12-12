use crate::events::{Event, EventType::*};
use crate::{ClientID, Money, Projector, TransactionID};
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
pub struct AccountProjector {
    accounts: BTreeMap<ClientID, Account>,
    deposits: BTreeMap<TransactionID, (ClientID, Money)>,
}

impl AccountProjector {
    pub fn new() -> Self {
        Self {
            accounts: Default::default(),
            deposits: Default::default(),
        }
    }

    pub fn stream_csv<R: std::io::Read, W: std::io::Write>(reader: R, writer: W) -> Result<()> {
        let accounts = Self::read_csv(reader)?;
        accounts.write_csv(writer)
    }

    pub fn read_csv<R: std::io::Read>(reader: R) -> Result<Self> {
        let mut reader = csv::ReaderBuilder::new().trim(csv::Trim::All).from_reader(reader);
        let mut accounts = Self::new();
        for event in reader.deserialize() {
            let result = event
                .context("Failed to deserialize a CSV record")
                .and_then(|e| accounts.project(&e));
            if let Err(err) = result {
                eprintln!("{err}");
            }
        }
        Ok(accounts)
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

impl Projector for AccountProjector {
    fn project(&mut self, event: &Event) -> Result<()> {
        let account = self.accounts.entry(event.client).or_default();
        if account.locked {
            return Err(anyhow!("Account locked: {}", event.client));
        }
        let mut amount = event.amount.unwrap_or_default();

        if matches!(event._type, Dispute | Resolve | Chargeback) {
            let deposit = self
                .deposits
                .get(&event.tx)
                .ok_or_else(|| anyhow!("Transaction not found; skipping."))?;
            if deposit.0 != event.client {
                return Err(anyhow!("Transaction not owned by client: {}", event.tx));
            }
            amount = deposit.1;
        }

        match event._type {
            Deposit => {
                self.deposits.insert(event.tx, (event.client, amount));
                account.available += amount;
            }
            Withdrawal => {
                if account.available >= amount {
                    account.available -= amount;
                } else {
                    return Err(anyhow!("Insufficient funds for withdrawal: {}", amount));
                }
            }
            Dispute => {
                if account.disputed.contains(&event.tx) {
                    return Err(anyhow!("Transaction already disputed: {}", event.tx));
                }
                account.disputed.insert(event.tx);
                account.available -= amount;
                account.held += amount;
            }
            Resolve => {
                if !account.disputed.contains(&event.tx) {
                    return Err(anyhow!("Transaction not disputed: {}", event.tx));
                }
                account.disputed.remove(&event.tx);
                account.available += amount;
                account.held -= amount;
            }
            Chargeback => {
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
