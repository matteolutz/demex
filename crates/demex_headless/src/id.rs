use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DemexProtoDeviceId {
    Controller,
    Node(u32),
}
