use crate::domain::events::AccountEvent;
use crate::domain::fund::{FundAccount, FundTransaction, TransactionType};
use crate::infrastructure::nats::EventPublisher;
use crate::infrastructure::repositories::fund_repository::{
    FundAccountRepository, FundTransactionRepository,
};
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;

/// Application service for fund-related use cases
pub struct FundApplicationService {
    account_repo: Arc<dyn FundAccountRepository>,
    tx_repo: Arc<dyn FundTransactionRepository>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl FundApplicationService {
    pub fn new(
        account_repo: Arc<dyn FundAccountRepository>,
        tx_repo: Arc<dyn FundTransactionRepository>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            account_repo,
            tx_repo,
            event_publisher,
        }
    }

    pub async fn get_balance(&self, user_id: uuid::Uuid, currency: &str) -> Result<FundAccount> {
        self.account_repo
            .find_by_user_and_currency(user_id, currency)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Fund account not found"))
    }

    pub async fn deposit(
        &self,
        user_id: uuid::Uuid,
        amount: i64,
        currency: &str,
        remark: &str,
    ) -> Result<FundTransaction> {
        let mut account = self
            .account_repo
            .find_by_user_and_currency(user_id, currency)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Fund account not found"))?;

        let balance_before = account.balance;
        account.deposit(amount)?;
        let saved = self.account_repo.update_balance(&account).await?;

        let tx = FundTransaction {
            id: uuid::Uuid::now_v7(),
            user_id,
            account_id: saved.id,
            transaction_type: TransactionType::Deposit,
            amount,
            balance_before,
            balance_after: saved.balance,
            order_id: None,
            remark: remark.to_string(),
            created_at: Utc::now(),
        };
        let saved_tx = self.tx_repo.create(&tx).await?;

        self.event_publisher
            .publish(&AccountEvent::FundChanged {
                user_id,
                account_id: saved.id,
                transaction_type: "DEPOSIT".to_string(),
                amount,
                balance_after: saved.balance,
                timestamp: Utc::now(),
            })
            .await?;

        Ok(saved_tx)
    }

    pub async fn withdraw(
        &self,
        user_id: uuid::Uuid,
        amount: i64,
        currency: &str,
        remark: &str,
    ) -> Result<FundTransaction> {
        let mut account = self
            .account_repo
            .find_by_user_and_currency(user_id, currency)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Fund account not found"))?;

        let balance_before = account.balance;
        account.withdraw(amount)?;
        let saved = self.account_repo.update_balance(&account).await?;

        let tx = FundTransaction {
            id: uuid::Uuid::now_v7(),
            user_id,
            account_id: saved.id,
            transaction_type: TransactionType::Withdrawal,
            amount: -amount,
            balance_before,
            balance_after: saved.balance,
            order_id: None,
            remark: remark.to_string(),
            created_at: Utc::now(),
        };
        let saved_tx = self.tx_repo.create(&tx).await?;

        self.event_publisher
            .publish(&AccountEvent::FundChanged {
                user_id,
                account_id: saved.id,
                transaction_type: "WITHDRAWAL".to_string(),
                amount: -amount,
                balance_after: saved.balance,
                timestamp: Utc::now(),
            })
            .await?;

        Ok(saved_tx)
    }

    pub async fn freeze(
        &self,
        user_id: uuid::Uuid,
        amount: i64,
        currency: &str,
        order_id: &str,
    ) -> Result<FundTransaction> {
        let mut account = self
            .account_repo
            .find_by_user_and_currency(user_id, currency)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Fund account not found"))?;

        let balance_before = account.balance;
        let frozen_before = account.frozen_balance;
        account.freeze(amount)?;
        let saved = self.account_repo.update_balance(&account).await?;

        let tx = FundTransaction {
            id: uuid::Uuid::now_v7(),
            user_id,
            account_id: saved.id,
            transaction_type: TransactionType::FundFrozen,
            amount,
            balance_before: balance_before - frozen_before,
            balance_after: saved.balance - saved.frozen_balance,
            order_id: Some(order_id.to_string()),
            remark: format!("Freeze for order {}", order_id),
            created_at: Utc::now(),
        };
        let saved_tx = self.tx_repo.create(&tx).await?;

        self.event_publisher
            .publish(&AccountEvent::FundChanged {
                user_id,
                account_id: saved.id,
                transaction_type: "FUND_FROZEN".to_string(),
                amount,
                balance_after: saved.balance,
                timestamp: Utc::now(),
            })
            .await?;

        Ok(saved_tx)
    }

    pub async fn unfreeze(
        &self,
        user_id: uuid::Uuid,
        amount: i64,
        currency: &str,
        order_id: &str,
    ) -> Result<FundTransaction> {
        let mut account = self
            .account_repo
            .find_by_user_and_currency(user_id, currency)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Fund account not found"))?;

        let balance_before = account.balance;
        let frozen_before = account.frozen_balance;
        account.unfreeze(amount)?;
        let saved = self.account_repo.update_balance(&account).await?;

        let tx = FundTransaction {
            id: uuid::Uuid::now_v7(),
            user_id,
            account_id: saved.id,
            transaction_type: TransactionType::FundUnfrozen,
            amount: -amount,
            balance_before: balance_before - frozen_before,
            balance_after: saved.balance - saved.frozen_balance,
            order_id: Some(order_id.to_string()),
            remark: format!("Unfreeze for order {}", order_id),
            created_at: Utc::now(),
        };
        let saved_tx = self.tx_repo.create(&tx).await?;

        self.event_publisher
            .publish(&AccountEvent::FundChanged {
                user_id,
                account_id: saved.id,
                transaction_type: "FUND_UNFROZEN".to_string(),
                amount: -amount,
                balance_after: saved.balance,
                timestamp: Utc::now(),
            })
            .await?;

        Ok(saved_tx)
    }

    pub async fn get_transactions(
        &self,
        user_id: uuid::Uuid,
        page: i32,
        page_size: i32,
    ) -> Result<(Vec<FundTransaction>, i64)> {
        let offset = ((page - 1).max(0) * page_size) as i64;
        self.tx_repo
            .find_by_user(user_id, page_size as i64, offset)
            .await
            .map_err(|e| e.into())
    }
}
