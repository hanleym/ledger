use crate::events::Event;
use anyhow::{Result, anyhow};
use rust_decimal::Decimal;

mod accounts;
mod events;
#[cfg(test)]
mod tests;

type ClientID = u16;
type TransactionID = u32;
type Money = Decimal;

pub trait Projector {
    fn project(&mut self, event: &Event) -> Result<()>;
}

// fn project(projectors: &mut [&mut dyn Projector], event: &Event) -> Result<()> {
//     for projector in projectors {
//         projector.project(event)?;
//     }
//     Ok(())
// }

fn main() -> Result<()> {
    let path = std::env::args().nth(1).ok_or(anyhow!("No file provided"))?;
    let file = std::fs::File::open(path)?;
    accounts::AccountProjector::stream_csv(file, std::io::stdout())?;
    Ok(())
}
