use serde::{Serialize, Deserialize};

#[derive(Debug,Serialize, Deserialize)]
pub struct CurrencyUsdPrice {
	timestamp: u64,
    #[serde(rename = "BTC")]
	btc: f64,
    #[serde(rename = "ETH")]
	eth: f64,
    #[serde(rename = "BNB")]
	bnb: f64,
    #[serde(rename = "POL")]
	pol: f64,
}
