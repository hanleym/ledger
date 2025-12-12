use crate::Money;
use crate::balance_sheet::{BalanceSheet, Output};
use crate::event_store::{Event, EventStore};
use anyhow::Result;
use std::io::Cursor;

#[test]
fn test_csv() -> Result<()> {
    let file = include_str!("../tests/transactions.csv");
    let event_store = EventStore::from_csv(Cursor::new(file))?;
    let mut balance_sheet = BalanceSheet::new();
    event_store.project(&mut [&mut balance_sheet])?;
    let mut buff = Vec::new();
    balance_sheet.write_csv(&mut buff)?;
    let output = String::from_utf8(buff)?;
    let expected = include_str!("../tests/accounts.csv");
    assert_eq!(output, expected);
    Ok(())
}

fn table_test(events: Vec<Event>, expected: Vec<Output>) -> Result<()> {
    let mut event_store = EventStore::new();
    for event in events {
        event_store.save(event)?;
    }
    let mut balance_sheet = BalanceSheet::new();
    event_store.project(&mut [&mut balance_sheet])?;
    let got = balance_sheet.into_output();
    assert_eq!(got, expected);
    Ok(())
}

#[test]
fn test_basic() -> Result<()> {
    table_test(
        vec![
            Event::deposit(1, 1, 1),
            Event::deposit(2, 2, 2),
            Event::deposit(1, 3, 2),
            Event::withdraw(1, 4, Money::new(15, 1)),
            Event::withdraw(1, 5, 3),
        ],
        vec![
            Output::new(1, Money::new(15, 1), 0, false),
            Output::new(2, 2, 0, false),
        ],
    )
}

#[test]
fn test_insufficient_funds() -> Result<()> {
    table_test(
        vec![Event::deposit(1, 1, 1), Event::withdraw(1, 2, 2)],
        vec![Output::new(1, 1, 0, false)],
    )
}

#[test]
fn test_dispute() -> Result<()> {
    table_test(
        vec![Event::deposit(1, 1, 5), Event::dispute(1, 1)],
        vec![Output::new(1, 0, 5, false)],
    )
}

#[test]
fn test_duplicated_dispute() -> Result<()> {
    table_test(
        vec![
            Event::deposit(1, 1, 5),
            Event::dispute(1, 1),
            Event::dispute(1, 1),
        ],
        vec![Output::new(1, 0, 5, false)],
    )
}

#[test]
fn test_resolve() -> Result<()> {
    table_test(
        vec![
            Event::deposit(1, 1, 5),
            Event::dispute(1, 1),
            Event::resolve(1, 1),
        ],
        vec![Output::new(1, 5, 0, false)],
    )
}

#[test]
fn test_duplicated_resolve() -> Result<()> {
    table_test(
        vec![
            Event::deposit(1, 1, 5),
            Event::dispute(1, 1),
            Event::resolve(1, 1),
            Event::resolve(1, 1),
        ],
        vec![Output::new(1, 5, 0, false)],
    )
}

#[test]
fn test_chargeback() -> Result<()> {
    table_test(
        vec![
            Event::deposit(1, 1, 5),
            Event::dispute(1, 1),
            Event::chargeback(1, 1),
        ],
        vec![Output::new(1, 0, 0, true)],
    )
}

#[test]
fn test_duplicated_chargeback() -> Result<()> {
    table_test(
        vec![
            Event::deposit(1, 1, 5),
            Event::dispute(1, 1),
            Event::chargeback(1, 1),
            Event::chargeback(1, 1),
        ],
        vec![Output::new(1, 0, 0, true)],
    )
}

#[test]
fn test_locking() -> Result<()> {
    table_test(
        vec![
            Event::deposit(1, 1, 5),
            Event::dispute(1, 1),
            Event::chargeback(1, 1),
            Event::deposit(1, 1, 5),
            Event::deposit(1, 1, 3),
        ],
        vec![Output::new(1, 0, 0, true)],
    )
}
