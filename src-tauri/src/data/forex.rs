
use serde::{Serialize, Deserialize};

/// 法币汇率（相对 USD 或其它主货币）
#[derive(Debug, Serialize, Deserialize)]
pub struct FiatRates {
    /// 时间戳（秒）
    pub timestamp: f64,

    // --- 北美 / 欧洲 ---
    #[serde(rename = "USD")]  // 美元
    pub usd: f64,
    #[serde(rename = "EUR")]  // 欧元
    pub eur: f64,
    #[serde(rename = "GBP")]  // 英镑
    pub gbp: f64,
    #[serde(rename = "CHF")]  // 瑞士法郎
    pub chf: f64,
    #[serde(rename = "CAD")]  // 加拿大元
    pub cad: f64,
    #[serde(rename = "AUD")]  // 澳大利亚元
    pub aud: f64,
    #[serde(rename = "NZD")]  // 新西兰元
    pub nzd: f64,

    // --- 东亚 ---
    #[serde(rename = "JPY")]  // 日元
    pub jpy: f64,
    #[serde(rename = "CNY")]  // 人民币
    pub cny: f64,
    #[serde(rename = "KRW")]  // 韩元
    pub krw: f64,

    // --- 东南亚 ---
    #[serde(rename = "SGD")]  // 新加坡元
    pub sgd: f64,
    #[serde(rename = "VND")]  // 越南盾
    pub vnd: f64,
    #[serde(rename = "MYR")]  // 马来西亚林吉特
    pub myr: f64,
    #[serde(rename = "IDR")]  // 印尼盾
    pub idr: f64,
    #[serde(rename = "THB")]  // 泰铢
    pub thb: f64,
    #[serde(rename = "PHP")]  // 菲律宾比索
    pub php: f64,

    // --- 南亚 ---
    #[serde(rename = "INR")]  // 印度卢比
    pub inr: f64,
    #[serde(rename = "PKR")]  // 巴基斯坦卢比
    pub pkr: f64,

    // --- 南美 ---
    #[serde(rename = "VES")]  // 委内瑞拉玻利瓦尔
    pub ves: f64,
    #[serde(rename = "ARS")]  // 阿根廷比索
    pub ars: f64,
    #[serde(rename = "BRL")]  // 巴西雷亚尔
    pub brl: f64,
    #[serde(rename = "CLP")]  // 智利比索
    pub clp: f64,
    #[serde(rename = "COP")]  // 哥伦比亚比索
    pub cop: f64,
    #[serde(rename = "PEN")]  // 秘鲁新索尔
    pub pen: f64,
}
