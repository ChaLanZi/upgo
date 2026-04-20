use crate::application::RiskApplicationService;
use std::sync::Arc;

/// gRPC handler for RiskService
pub struct RiskGrpcHandler {
    #[allow(dead_code)]
    risk_service: Arc<RiskApplicationService>,
}

impl RiskGrpcHandler {
    pub fn new(risk_service: Arc<RiskApplicationService>) -> Self {
        Self { risk_service }
    }
}
