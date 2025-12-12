use crate::accounts::AccountProjector;
use anyhow::Result;

fn table_test(transactions: &str, expected: &str) -> Result<()> {
    let mut got = Vec::new();
    AccountProjector::stream_csv(std::io::Cursor::new(transactions), &mut got)?;
    let got = String::from_utf8(got)?;
    assert_eq!(got, expected);
    Ok(())
}

#[test]
fn test_basic() -> Result<()> {
    table_test(
        r#"type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0
deposit, 1, 3, 2.0
withdrawal, 1, 4, 1.5
withdrawal, 2, 5, 3.0
"#,
        r#"client,available,held,total,locked
1,1.5,0,1.5,false
2,2,0,2,false
"#,
    )
}

#[test]
fn test_insufficient_funds() -> Result<()> {
    table_test(
        r#"type, client, tx, amount
deposit, 1, 1, 1
withdrawal, 1, 2, 2
"#,
        r#"client,available,held,total,locked
1,1,0,1,false
"#,
    )
}

#[test]
fn test_dispute() -> Result<()> {
    table_test(
        r#"type, client, tx, amount
deposit, 1, 1, 5
dispute, 1, 1,
"#,
        r#"client,available,held,total,locked
1,0,5,5,false
"#,
    )
}

#[test]
fn test_duplicated_dispute() -> Result<()> {
    table_test(
        r#"type, client, tx, amount
deposit, 1, 1, 5
dispute, 1, 1,
dispute, 1, 1,
"#,
        r#"client,available,held,total,locked
1,0,5,5,false
"#,
    )
}

#[test]
fn test_dispute_other_client_transaction() -> Result<()> {
    table_test(
        r#"type, client, tx, amount
deposit, 1, 1, 5
dispute, 2, 1,
"#,
        r#"client,available,held,total,locked
1,5,0,5,false
2,0,0,0,false
"#,
    )
}

#[test]
fn test_resolve() -> Result<()> {
    table_test(
        r#"type, client, tx, amount
deposit, 1, 1, 5
dispute, 1, 1,
resolve, 1, 1,
"#,
        r#"client,available,held,total,locked
1,5,0,5,false
"#,
    )
}

#[test]
fn test_duplicated_resolve() -> Result<()> {
    table_test(
        r#"type, client, tx, amount
deposit, 1, 1, 5
dispute, 1, 1,
resolve, 1, 1,
resolve, 1, 1,
"#,
        r#"client,available,held,total,locked
1,5,0,5,false
"#,
    )
}

#[test]
fn test_chargeback() -> Result<()> {
    table_test(
        r#"type, client, tx, amount
deposit, 1, 1, 5
dispute, 1, 1,
chargeback, 1, 1,
"#,
        r#"client,available,held,total,locked
1,0,0,0,true
"#,
    )
}

#[test]
fn test_duplicated_chargeback() -> Result<()> {
    table_test(
        r#"type, client, tx, amount
deposit, 1, 1, 5
dispute, 1, 1,
chargeback, 1, 1,
chargeback, 1, 1,
"#,
        r#"client,available,held,total,locked
1,0,0,0,true
"#,
    )
}

#[test]
fn test_locking() -> Result<()> {
    table_test(
        r#"type, client, tx, amount
deposit, 1, 1, 5
dispute, 1, 1,
chargeback, 1, 1,
deposit, 1, 2, 5
"#,
        r#"client,available,held,total,locked
1,0,0,0,true
"#,
    )
}
