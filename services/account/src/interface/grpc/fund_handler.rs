use crate::application::FundApplicationService;
use std::sync::Arc;

/// gRPC handler for FundService
pub struct FundGrpcHandler {
    #[allow(dead_code)]
    fund_service: Arc<FundApplicationService>,
}

impl FundGrpcHandler {
    pub fn new(fund_service: Arc<FundApplicationService>) -> Self {
        Self { fund_service }
    }
}
