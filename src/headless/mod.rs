pub mod controller;
pub mod error;
pub mod id;
pub mod node;
pub mod packet;
pub mod protocol;
pub mod sync;

const DEMEX_HEADLESS_TCP_PORT: u16 = 4545;
const DEMEX_HEADLESS_CONTROLLER_UDP_PORT: u16 = 4546;
const DEMEX_HEADLESS_NODE_UDP_PORT: u16 = 4547;
