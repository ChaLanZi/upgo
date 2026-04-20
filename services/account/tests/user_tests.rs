mod common;

use account::application::UserApplicationService;
use account::domain::user::{AccountStatus, KycStatus, UserId};
use std::sync::Arc;

fn make_service() -> UserApplicationService {
    let user_repo = Arc::new(common::InMemoryUserRepository::new());
    let publisher = Arc::new(common::InMemoryEventPublisher::new());
    UserApplicationService::new(user_repo, publisher)
}

#[tokio::test]
async fn test_register_success() {
    let svc = make_service();
    let user = svc
        .register(
            "alice@example.com".into(),
            "hash123".into(),
            "Alice".into(),
            None,
        )
        .await
        .unwrap();
    assert_eq!(user.email, "alice@example.com");
    assert_eq!(user.nickname, "Alice");
    assert_eq!(user.account_status, AccountStatus::PendingVerification);
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let svc = make_service();
    svc.register("dup@example.com".into(), "h1".into(), "U1".into(), None)
        .await
        .unwrap();
    let result = svc
        .register("dup@example.com".into(), "h2".into(), "U2".into(), None)
        .await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Email already exists"));
}

#[tokio::test]
async fn test_get_profile_not_found() {
    let svc = make_service();
    let result = svc.get_profile(&UserId::new()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_profile_after_register() {
    let svc = make_service();
    let user = svc
        .register("bob@example.com".into(), "hash".into(), "Bob".into(), None)
        .await
        .unwrap();
    let found = svc.get_profile(&user.id).await.unwrap();
    assert_eq!(found.email, "bob@example.com");
}

#[tokio::test]
async fn test_kyc_full_lifecycle() {
    let svc = make_service();
    let user = svc
        .register(
            "kyc@example.com".into(),
            "hash".into(),
            "KYCUser".into(),
            None,
        )
        .await
        .unwrap();
    assert_eq!(user.kyc_status, KycStatus::None);

    let submitted = svc.submit_kyc(&user.id).await.unwrap();
    assert_eq!(submitted.kyc_status, KycStatus::PendingReview);

    let approved = svc.approve_kyc(&user.id).await.unwrap();
    assert_eq!(approved.kyc_status, KycStatus::Verified);
    assert!(approved.kyc_status.can_trade());

    let re_submit = svc.submit_kyc(&user.id).await;
    assert!(re_submit.is_err());
}

#[tokio::test]
async fn test_kyc_reject_and_retry() {
    let svc = make_service();
    let user = svc
        .register("kyc2@example.com".into(), "hash".into(), "U".into(), None)
        .await
        .unwrap();
    svc.submit_kyc(&user.id).await.unwrap();
    let rejected = svc.reject_kyc(&user.id).await.unwrap();
    assert_eq!(rejected.kyc_status, KycStatus::Rejected);

    let retry = svc.submit_kyc(&user.id).await.unwrap();
    assert_eq!(retry.kyc_status, KycStatus::PendingReview);
}

#[tokio::test]
async fn test_update_profile() {
    let svc = make_service();
    let user = svc
        .register(
            "upd@example.com".into(),
            "hash".into(),
            "OldName".into(),
            None,
        )
        .await
        .unwrap();
    let updated = svc
        .update_profile(&user.id, Some("NewName".into()), Some("13800138000".into()))
        .await
        .unwrap();
    assert_eq!(updated.nickname, "NewName");
    assert_eq!(updated.phone, Some("13800138000".into()));
}
