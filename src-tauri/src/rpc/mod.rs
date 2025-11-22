pub mod https;
pub mod multicall3;
pub mod ankr;
pub mod public;
pub mod gateway1;
pub mod method;
pub mod gateway;


pub enum RpcMode {
    Custom,
    Public,
    Gateway
}

pub enum RpcConnectionMode {
    Https,
    Wss,
    Helios,
}
