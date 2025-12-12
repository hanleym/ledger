use anyhow::{Result, anyhow};
use rust_decimal::Decimal;

mod balance_sheet;
mod event_store;
#[cfg(test)]
mod tests;

type ClientID = u16;
type TransactionID = u32;
type Money = Decimal;

fn main() -> Result<()> {
    env_logger::init();
    let path = std::env::args().nth(1).ok_or(anyhow!("No file provided"))?;
    let file = std::fs::File::open(path)?;
    let event_store = event_store::EventStore::from_csv(file)?;
    let mut balance_sheet = balance_sheet::BalanceSheet::new();
    event_store.project(&mut [&mut balance_sheet])?;
    balance_sheet.write_csv(std::io::stdout())?;
    Ok(())
}
