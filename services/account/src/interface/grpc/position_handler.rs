use crate::application::PositionApplicationService;
use std::sync::Arc;

/// gRPC handler for PositionService
pub struct PositionGrpcHandler {
    #[allow(dead_code)]
    position_service: Arc<PositionApplicationService>,
}

impl PositionGrpcHandler {
    pub fn new(position_service: Arc<PositionApplicationService>) -> Self {
        Self { position_service }
    }
}
