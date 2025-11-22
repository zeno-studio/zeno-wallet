// 测试文件用于验证 Helios 模块是否正确实现
use crate::helios::client::{init_helios, HeliosClient};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helios_client_type() {
        // 这只是一个编译时测试，确保类型正确
        let _client_type: HeliosClient;
    }
}