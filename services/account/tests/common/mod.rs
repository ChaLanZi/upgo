use account::domain::events::AccountEvent;
use account::domain::fund::{FundAccount, FundTransaction};
use account::domain::user::{User, UserId};
use account::infrastructure::nats::EventPublisher;
use account::infrastructure::repositories::fund_repository::{
    FundAccountRepository, FundTransactionRepository,
};
use account::infrastructure::repositories::user_repository::UserRepository;

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// InMemoryUserRepository
// ---------------------------------------------------------------------------

pub struct InMemoryUserRepository {
    users: Mutex<HashMap<String, User>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self {
            users: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, sqlx::Error> {
        let users = self.users.lock().unwrap();
        Ok(users.values().find(|u| u.id.0 == id.0).cloned())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        let users = self.users.lock().unwrap();
        Ok(users.get(email).cloned())
    }

    async fn create(&self, user: &User) -> Result<User, sqlx::Error> {
        let mut users = self.users.lock().unwrap();
        users.insert(user.email.clone(), user.clone());
        Ok(user.clone())
    }

    async fn update(&self, user: &User) -> Result<User, sqlx::Error> {
        let mut users = self.users.lock().unwrap();
        users.insert(user.email.clone(), user.clone());
        Ok(user.clone())
    }

    async fn exists_by_email(&self, email: &str) -> Result<bool, sqlx::Error> {
        let users = self.users.lock().unwrap();
        Ok(users.contains_key(email))
    }
}

// ---------------------------------------------------------------------------
// InMemoryFundAccountRepository
// ---------------------------------------------------------------------------

pub struct InMemoryFundAccountRepository {
    accounts: Mutex<Vec<FundAccount>>,
}

impl InMemoryFundAccountRepository {
    pub fn new() -> Self {
        Self {
            accounts: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl FundAccountRepository for InMemoryFundAccountRepository {
    async fn find_by_user_and_currency(
        &self,
        user_id: Uuid,
        currency: &str,
    ) -> Result<Option<FundAccount>, sqlx::Error> {
        let accounts = self.accounts.lock().unwrap();
        Ok(accounts
            .iter()
            .find(|a| a.user_id == user_id && a.currency.as_str() == currency)
            .cloned())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<FundAccount>, sqlx::Error> {
        let accounts = self.accounts.lock().unwrap();
        Ok(accounts.iter().find(|a| a.id == id).cloned())
    }

    async fn create(&self, account: &FundAccount) -> Result<FundAccount, sqlx::Error> {
        let mut accounts = self.accounts.lock().unwrap();
        accounts.push(account.clone());
        Ok(account.clone())
    }

    async fn update_balance(&self, account: &FundAccount) -> Result<FundAccount, sqlx::Error> {
        let mut accounts = self.accounts.lock().unwrap();
        if let Some(existing) = accounts.iter_mut().find(|a| a.id == account.id) {
            existing.balance = account.balance;
            existing.frozen_balance = account.frozen_balance;
            existing.version += 1;
            Ok(existing.clone())
        } else {
            Err(sqlx::Error::Protocol("not found".to_string()))
        }
    }
}

// ---------------------------------------------------------------------------
// InMemoryFundTransactionRepository
// ---------------------------------------------------------------------------

pub struct InMemoryFundTransactionRepository {
    txs: Mutex<Vec<FundTransaction>>,
}

impl InMemoryFundTransactionRepository {
    pub fn new() -> Self {
        Self {
            txs: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl FundTransactionRepository for InMemoryFundTransactionRepository {
    async fn create(&self, tx: &FundTransaction) -> Result<FundTransaction, sqlx::Error> {
        let mut txs = self.txs.lock().unwrap();
        txs.push(tx.clone());
        Ok(tx.clone())
    }

    async fn find_by_user(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<FundTransaction>, i64), sqlx::Error> {
        let txs = self.txs.lock().unwrap();
        let user_txs: Vec<_> = txs
            .iter()
            .filter(|t| t.user_id == user_id)
            .cloned()
            .collect();
        let total = user_txs.len() as i64;
        let page = user_txs
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect();
        Ok((page, total))
    }
}

// ---------------------------------------------------------------------------
// InMemoryEventPublisher
// ---------------------------------------------------------------------------

pub struct InMemoryEventPublisher {
    pub events: Mutex<Vec<AccountEvent>>,
}

impl InMemoryEventPublisher {
    pub fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl EventPublisher for InMemoryEventPublisher {
    async fn publish(&self, event: &AccountEvent) -> anyhow::Result<()> {
        let mut events = self.events.lock().unwrap();
        events.push(event.clone());
        Ok(())
    }
}
