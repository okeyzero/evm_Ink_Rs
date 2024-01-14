use std::{env, process};

use ethers::prelude::U256;
use ethers::utils::{hex, parse_units};
use log::error;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default = "default_prefix")]
    pub prefix: String,
    pub rpc_url: String,
    pub private_key: String,
    #[serde(skip_deserializing)]
    pub address: String,
    pub to_address: Option<String>,
    pub max_fee_per_gas: f64,
    pub max_priority_fee_per_gas: Option<f64>,
    #[serde(default = "default_gas_limit")]
    pub gas_limit: u64,
    pub count: u64,
    pub data: String,
    #[serde(skip_deserializing)]
    pub hex_text: Option<String>,
    #[serde(skip_deserializing)]
    pub id: Option<crate::Id>,
    #[serde(default = "default_value")]
    pub value: f64,
    #[serde(default = "default_batch_size")]
    pub batch_size: u64,
    #[serde(default = "default_interval")]
    pub interval: f64,
}
fn default_prefix() -> String {
    "data:,".to_string()
}
fn default_gas_limit() -> u64 {
    50000
}
fn default_value() -> f64 {
    0.0
}
fn default_batch_size() -> u64 {
    100
}
fn default_interval() -> f64 {
    0.0
}

#[derive(Debug, Clone)]
pub struct Id {
    pub id: u64,
    pub start_id: Option<u64>,
    pub end_id: Option<u64>,
    pub match_id: String,
}

#[derive(Debug)]
pub struct GasPrice {
    pub eip1559: bool,
    pub max_fee_per_gas: U256,
    pub max_priority_fee_per_gas: U256,
    pub value: U256,
}

impl crate::Config {
    pub fn get_hex_text(&mut self) -> String {
        if self.data.starts_with("0x") {
            self.data.clone()
        } else {
            let data = self.process_text();
            hex::encode_prefixed(format!("{}{}", self.prefix, data).as_bytes())
        }
    }

    pub fn process_text(&mut self) -> String {
        let mut text = self.data.replace("[address]", &self.address);
        if let Some(id) = &self.id {
            text = text.replace(&id.match_id, &id.id.to_string());
            self.auto_set_id(); // 确保在每次调用 process_text 时调用 auto_set_id
        }
        text
    }

    pub fn set_id(&mut self, _id: u64) {
        let mut id = self.id.clone().unwrap();
        id.id = _id;
        self.id = Some(id);
    }

    fn auto_set_id(&mut self) {
        let id = self.id.as_ref().unwrap();
        match id.start_id {
            Some(_) => {
                self.set_id(id.id + 1);
            }
            None => {
                self.set_id(id.id - 1);
            }
        }
    }

    pub fn init_gas_price(&self) -> crate::GasPrice {
        let max_fee_per_gas = U256::from(parse_units(self.max_fee_per_gas, "gwei").unwrap());
        let max_priority_fee_per_gas = match self.max_priority_fee_per_gas {
            Some(priority_fee) => U256::from(parse_units(priority_fee, "gwei").unwrap()),
            None => U256::from(0),
        };
        let value = U256::from(parse_units(self.value, "ether").unwrap());

        crate::GasPrice {
            eip1559: self.max_priority_fee_per_gas.is_some(),
            max_fee_per_gas,
            max_priority_fee_per_gas,
            value,
        }
    }
}

pub fn execution_addresses(config: Config) -> Vec<Config> {
    if let Some(wallets_file) = env::var("wallets_file").ok().filter(|s| !s.is_empty()) {
        let wallets = std::fs::read_to_string(&wallets_file).unwrap_or_else(|_| {
            error!("读取文件失败: {}", wallets_file);
            process::exit(1);
        });

        let wallets: Vec<Config> = wallets
            .lines()
            .map(|line| {
                let parts: Vec<&str> = line.split("----").collect();
                let private_key = parts[parts.len() - 1].trim();
                let mut config = config.clone();
                config.private_key = private_key.to_string();
                config
            })
            .collect();

        wallets
    } else {
        vec![config]
    }
}

pub fn decode_hex(hex: &str) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = hex::decode(hex)?;
    let text = String::from_utf8(bytes)?;
    Ok(text)
}

pub fn process_id(text: &str) -> (Option<Id>, Option<u64>, u64) {
    let re = regex::Regex::new(r"\[(\d+)?-(\d+)?]").unwrap();
    if let Some(caps) = re.captures(&text) {
        let match_id = caps.get(0).unwrap().as_str().to_string();
        let start_id: Option<u64> = caps.get(1).and_then(|m| m.as_str().parse().ok());
        let end_id: Option<u64> = caps.get(2).and_then(|m| m.as_str().parse().ok());
        // 如果 start_id 不为 None 设置id 为 start_id 否则设置 id 为 end_id
        if start_id.is_none() {
            (
                Some(Id {
                    id: end_id.unwrap(),
                    start_id,
                    end_id,
                    match_id,
                }),
                end_id,
                u64::MAX,
            )
        } else {
            (
                Some(Id {
                    id: start_id.unwrap(),
                    start_id,
                    end_id,
                    match_id,
                }),
                start_id,
                if end_id.is_none() {
                    u64::MAX
                } else {
                    end_id.unwrap() - start_id.unwrap() + 1
                },
            )
        }
    } else {
        (None, None, u64::MAX)
    }
}
