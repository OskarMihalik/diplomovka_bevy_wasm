use dto::auth::LoginDto;
use goose::prelude::*;

async fn loadtest_index(user: &mut GooseUser) -> TransactionResult {
    let body = LoginDto {
        email: "user@mail.com".to_string(),
        password: "user@mail.com".to_string(),
    };
    let _goose_metrics = user.post_json("/login", &body).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), GooseError> {
    GooseAttack::initialize()?
        .register_scenario(
            scenario!("LoadtestTransactions").register_transaction(transaction!(loadtest_index)),
        )
        .set_default(GooseDefault::Host, "http://localhost:4000/")?
        .set_default(GooseDefault::ReportFile, "axumReport.html")?
        .set_default(GooseDefault::NoResetMetrics, true)?
        .execute()
        .await?;

    GooseAttack::initialize()?
        .register_scenario(
            scenario!("LoadtestTransactions").register_transaction(transaction!(loadtest_index)),
        )
        .set_default(GooseDefault::Host, "http://localhost:4001/")?
        .set_default(GooseDefault::ReportFile, "expressReport.html")?
        .set_default(GooseDefault::NoResetMetrics, true)?
        .execute()
        .await?;

    Ok(())
}
