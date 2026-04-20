mod common;

use account::application::FundApplicationService;
use account::domain::fund::{Currency, FundAccount, TransactionType};
use account::infrastructure::repositories::fund_repository::FundAccountRepository;
use futures::FutureExt;
use std::sync::Arc;

fn make_service() -> (
    FundApplicationService,
    Arc<common::InMemoryFundAccountRepository>,
) {
    let account_repo = Arc::new(common::InMemoryFundAccountRepository::new());
    let publisher = Arc::new(common::InMemoryEventPublisher::new());
    let svc = FundApplicationService::new(
        account_repo.clone() as Arc<dyn FundAccountRepository>,
        Arc::new(common::InMemoryFundTransactionRepository::new()),
        publisher,
    );
    (svc, account_repo)
}

async fn seed(
    _svc: &FundApplicationService,
    repo: &common::InMemoryFundAccountRepository,
) -> uuid::Uuid {
    let uid = uuid::Uuid::now_v7();
    let acc = FundAccount::new(uid, Currency::CNY);
    repo.create(&acc).now_or_never().unwrap().unwrap();
    uid
}

#[tokio::test]
async fn test_deposit() {
    let (svc, repo) = make_service();
    let uid = seed(&svc, &repo).await;
    let tx = svc.deposit(uid, 10000, "CNY", "deposit").await.unwrap();
    assert_eq!(tx.transaction_type, TransactionType::Deposit);
    assert_eq!(tx.amount, 10000);

    let acc = svc.get_balance(uid, "CNY").await.unwrap();
    assert_eq!(acc.balance, 10000);
    assert_eq!(acc.available_balance(), 10000);
}

#[tokio::test]
async fn test_deposit_and_withdraw() {
    let (svc, repo) = make_service();
    let uid = seed(&svc, &repo).await;
    svc.deposit(uid, 50000, "CNY", "deposit").await.unwrap();
    svc.withdraw(uid, 20000, "CNY", "withdraw").await.unwrap();

    let acc = svc.get_balance(uid, "CNY").await.unwrap();
    assert_eq!(acc.balance, 30000);
}

#[tokio::test]
async fn test_withdraw_insufficient() {
    let (svc, repo) = make_service();
    let uid = seed(&svc, &repo).await;
    svc.deposit(uid, 1000, "CNY", "deposit").await.unwrap();
    let result = svc.withdraw(uid, 5000, "CNY", "overdraft").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_freeze_and_unfreeze() {
    let (svc, repo) = make_service();
    let uid = seed(&svc, &repo).await;
    svc.deposit(uid, 10000, "CNY", "deposit").await.unwrap();

    let frozen = svc.freeze(uid, 3000, "CNY", "ord-001").await.unwrap();
    assert_eq!(frozen.transaction_type, TransactionType::FundFrozen);

    let acc = svc.get_balance(uid, "CNY").await.unwrap();
    assert_eq!(acc.frozen_balance, 3000);
    assert_eq!(acc.available_balance(), 7000);

    svc.unfreeze(uid, 3000, "CNY", "ord-001").await.unwrap();
    let acc = svc.get_balance(uid, "CNY").await.unwrap();
    assert_eq!(acc.frozen_balance, 0);
    assert_eq!(acc.available_balance(), 10000);
}

#[tokio::test]
async fn test_transaction_history() {
    let (svc, repo) = make_service();
    let uid = seed(&svc, &repo).await;
    svc.deposit(uid, 10000, "CNY", "first").await.unwrap();
    svc.deposit(uid, 5000, "CNY", "second").await.unwrap();

    let (txs, total) = svc.get_transactions(uid, 1, 10).await.unwrap();
    assert_eq!(total, 2);
    assert_eq!(txs.len(), 2);
}

#[tokio::test]
async fn test_balance_not_found() {
    let (svc, _) = make_service();
    let result = svc.get_balance(uuid::Uuid::now_v7(), "CNY").await;
    assert!(result.is_err());
}
