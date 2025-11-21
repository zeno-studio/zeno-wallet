use ethers::abi::{AbiDecode, ParamType};

#[derive(Debug, Serialize)]
pub struct ParsedActivity {
    pub hash: String,
    pub activity_type: String,
    pub description: String,
    pub signature: Option<String>,
    pub standard: Option<String>,
}

pub async fn parse_ankr_activities(
    activities: &[Transaction],
) -> Vec<ParsedActivity> {
    let mut results = vec![];

    for tx in activities {
        let input = tx.input.clone().unwrap_or_default();

        // ========== Native Transfer ==========
        if tx.value != "0x0" && input == "0x" {
            let value = u128::from_str_radix(tx.value.trim_start_matches("0x"), 16)
                .unwrap_or(0) as f64 / 1e18;

            results.push(ParsedActivity {
                hash: tx.hash.clone(),
                activity_type: "Native Transfer".into(),
                description: format!("Transfer {} {} to {}", value, tx.blockchain, tx.to),
                signature: None,
                standard: None,
            });
            continue;
        }

        // ========== Contract Call - 无 input ==========
        if input == "0x" || input.is_empty() {
            results.push(ParsedActivity {
                hash: tx.hash.clone(),
                activity_type: "Contract Call".into(),
                description: "Unknown contract call".into(),
                signature: None,
                standard: None,
            });
            continue;
        }

        // ========== 4-byte selector ==========
        let selector = &input[0..10];

        // 示例：你可以根据自己的 function selector map 来匹配
        let sig = function_selector_lookup(selector);

        if sig.is_none() {
            results.push(ParsedActivity {
                hash: tx.hash.clone(),
                activity_type: "Contract Call".into(),
                description: "Unknown contract call".into(),
                signature: None,
                standard: None,
            });
            continue;
        }

        let sig_info = sig.unwrap();

        // ERC20 transfer(address,uint256)
        let description = if selector == "0xa9059cbb" {
            if let Ok(decoded) = ethers::abi::decode(
                &[ParamType::Address, ParamType::Uint(256)],
                &hex::decode(&input[10..]).unwrap_or_default()
            ) {
                let to = decoded[0].to_string();
                let amount = decoded[1].to_string();

                format!("ERC20 Transfer {} to {}", amount, to)
            } else {
                sig_info.description.clone()
            }
        } else {
            sig_info.description.clone()
        };

        results.push(ParsedActivity {
            hash: tx.hash.clone(),
            activity_type: format!("{} {}", sig_info.standard, sig_info.name),
            description,
            signature: Some(sig_info.signature.clone()),
            standard: Some(sig_info.standard.clone()),
        });
    }

    results
}
