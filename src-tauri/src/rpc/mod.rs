pub mod custom;
pub mod https;
pub mod multicall3;
pub mod ankr;


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
