use std::cmp::min;
use std::process;
use std::str::FromStr;
use std::time::Duration;
use dotenv::dotenv;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::prelude::*;
use ethers::utils::hex;
use ethers::utils::parse_units;
use ethers_batch_request::batch::{BatchRequest, BatchResponse};
use ethers_batch_request::middleware::BatchRequestMiddleware;
use log::{error, info, warn};
use serde::Deserialize;
use tokio;
use url::Url;

use crate::initialization::{log_banner, print_banner, setup_logger};

mod initialization;

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
    #[serde(default = "default_prefix_gas_limit")]
    pub gas_limit: u64,
    pub count: u64,
    pub data: String,
    #[serde(skip_deserializing)]
    pub hex_text: Option<String>,
    #[serde(skip_deserializing)]
    pub id: Option<Id>,
}
fn default_prefix() -> String {
    "data:,".to_string()
}
fn default_prefix_gas_limit() -> u64 {
    50000
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
    pub max_priority_fee_per_gas: U256
}

impl Config {

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
            self.auto_set_id();  // 确保在每次调用 process_text 时调用 auto_set_id
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

    pub fn init_gas_price(&self) -> GasPrice {
        let max_fee_per_gas = U256::from(parse_units(self.max_fee_per_gas, "gwei").unwrap());
        let max_priority_fee_per_gas = match self.max_priority_fee_per_gas {
            Some(priority_fee) => U256::from(parse_units(priority_fee, "gwei").unwrap()),
            None => U256::from(0),
        };

        GasPrice {
            eip1559: self.max_priority_fee_per_gas.is_some(),
            max_fee_per_gas,
            max_priority_fee_per_gas
        }
    }
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    setup_logger()?;
    print_banner();
    info!("开始执行任务");
    warn!("Author:[𝕏] @0xNaiXi");
    warn!("Author:[𝕏] @0xNaiXi");
    warn!("Author:[𝕏] @0xNaiXi");
    let mut config = envy::from_env::<Config>()?;
    let provider = Provider::<Http>::try_from(&config.rpc_url)?;
    let chain_id = provider.get_chainid().await?;
    let wallet = config.private_key.parse::<LocalWallet>()?.with_chain_id(chain_id.as_u64());
    let address = wallet.address();
    let nonce = provider.get_transaction_count(wallet.address(), None).await?;
    let client = BatchRequestMiddleware::new(provider, Url::parse(&config.rpc_url)?);
    let gas_price = config.init_gas_price();


    // 配置文件处理
    let (id, current_id, id_count) = process_id(&config.data);
    config.id = id;
    config.count = min(config.count, id_count);
    config.address = format!("{:?}", address);
    // 检查配置文件
    let to_address: Address = if let Some(str) = config.to_address.as_ref() {
        if str.is_empty() {
            address
        } else {
            str.parse()?
        }
    } else {
        address
    };
    config.to_address = Some(format!("{:?}", to_address));
    if config.data.is_empty()  {
        error!("data 不能为空");
        process::exit(1);
    }

    let data = config.get_hex_text();
    let text = decode_hex(&data)?;
    info!("当前链ID: {}", chain_id);
    info!("钱包地址: {:?}", address);
    info!("铭文接收地址: {:?}", to_address);
    info!("钱包nonce: {:?}", nonce);
    info!("mint 数据: {}", text);
    info!("十六进制数据: {}", data);
    info!("mint总数量: {}", config.count);
    if let Some(id) = current_id {
        config.set_id(id);
    }
    mint(&client, &wallet, config.clone(), &gas_price, nonce, to_address).await?;

    info!("任务执行完毕 程序将在 1000 秒后关闭");
    //编译成exe 取消下面的屏蔽 不让程序关闭窗口 不然的话 会执行完任务 直接关闭窗口 无法看输出的日志了
    tokio::time::sleep(Duration::new(1000, 0)).await;
    Ok(())
}

fn decode_hex(hex: &str) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = hex::decode(hex)?;
    let text = String::from_utf8(bytes)?;
    Ok(text)
}

fn process_id(text: &str) -> (Option<Id>, Option<u64>, u64) {
    let re = regex::Regex::new(r"\[(\d+)?-(\d+)?]").unwrap();
    if let Some(caps) = re.captures(&text) {
        let match_id = caps.get(0).unwrap().as_str().to_string();
        let start_id: Option<u64> = caps.get(1).and_then(|m| m.as_str().parse().ok());
        let end_id: Option<u64> = caps.get(2).and_then(|m| m.as_str().parse().ok());
        // 如果 start_id 不为 None 设置id 为 start_id 否则设置 id 为 end_id
        if start_id.is_none() {
            (Some(Id {
                id: end_id.unwrap(),
                start_id,
                end_id,
                match_id,
            }), end_id, u64::MAX)
        } else {
            (Some(Id {
                id: start_id.unwrap(),
                start_id,
                end_id,
                match_id,
            }), start_id, if end_id.is_none() { u64::MAX } else { end_id.unwrap() - start_id.unwrap() + 1 })
        }
    } else {
        (None, None, u64::MAX)
    }
}

async fn mint(provider: &BatchRequestMiddleware<Provider<Http>>, wallet: &Wallet<SigningKey>, mut config: Config, gas_price: &GasPrice, mut nonce: U256, to_address: Address) -> Result<bool, Box<dyn std::error::Error>> {
    let chain_id = wallet.chain_id();
    //每 100 为 一组 生成 100 个 tx
    let batch_size = 100;
    let batch_count = (config.count + batch_size - 1) / batch_size;
    for i in 0..batch_count {
        let start = i * batch_size;
        let end = min((i + 1) * batch_size, config.count);
        let current_batch_size = end - start; // 计算当前批次的实际大小
        log_banner(format!("第 {} 轮,共 {} 轮 当前批次大小 {}", i + 1, batch_count, current_batch_size));
        let mut batch = BatchRequest::with_capacity(current_batch_size as usize);
        for _ in start..end {
            let data = config.get_hex_text();
            //println!("data: {}", data);
            let data = Bytes::from_str(&data)?;
            //println!("data: {}", hex::encode(&data));
            let tx = if gas_price.eip1559 {
                Eip1559TransactionRequest::new()
                    .chain_id(chain_id)
                    .from(wallet.address())
                    .to(to_address)
                    .value(0)
                    .max_fee_per_gas(gas_price.max_fee_per_gas)
                    .max_priority_fee_per_gas(gas_price.max_priority_fee_per_gas)
                    .gas(config.gas_limit)
                    .nonce(nonce)
                    .data(data)
                    .access_list(vec![])
                    .into()
            } else {
                TransactionRequest::new()
                    .chain_id(chain_id)
                    .from(wallet.address())
                    .to(to_address)
                    .value(0)
                    .nonce(nonce)
                    .data(data)
                    .gas(config.gas_limit)
                    .gas_price(gas_price.max_fee_per_gas)
                    .into()
            };

            let signature = wallet.sign_transaction_sync(&tx)?;
            let signed_tx = tx.rlp_signed(&signature);

            let sign_tx = format!("0x{}", hex::encode(signed_tx));

            batch.add_request("eth_sendRawTransaction", vec![sign_tx])?;
            nonce = nonce + 1;
        }
        let mut http_responses: BatchResponse = provider.execute_batch(&mut batch).await?;
        let mut count = 0;

        while let Some(tx_response) = http_responses.next_response::<H256>() {
            match tx_response {
                Ok(tx_hash) => {
                    info!("第 {} 次 交易发送成功: {:?}",i * batch_size + count+1 ,tx_hash);
                }
                Err(e) => {
                    error!("第 {} 次 交易发送失败: {:?}",i * batch_size + count+1 ,e);
                }
            }
            count += 1;
        }
    }
    Ok(true)
}


#[cfg(test)]
mod tests {
    use std::env;
    use dotenv::dotenv;
    #[test]
    fn test(){
        dotenv().ok(); // 加载 .env 文件
        // 获取环境变量
        let json_str = env::var("data").expect("环境变量 MY_JSON 未设置");
        println!("json_str: {}", json_str);
    }

    #[test]
    fn regex_test() {
        let re = regex::Regex::new(r"\[(\d+)?-(\d+)?]").unwrap();
        let text = r#"{"p":"erc-20","op":"mint","tick":"pi","id":"6227","amt":"1000"}"#;
        println!("text: {}", text);
        if let Some(caps) = re.captures(&text) {
            let start_id: Option<u64> = caps.get(1).and_then(|m| m.as_str().parse().ok());
            let end_id: Option<u64> = caps.get(2).and_then(|m| m.as_str().parse().ok());
        } else {
            println!("未匹配到任何内容");
        }


        let text = r#"{"p":"erc-20","op":"mint","tick":"pi","id":"[1200-]","to":"[address]","amt":"1000"}"#;
        // start_id 可能为 u64 也可能为 None
        if let Some(caps) = re.captures(&text) {
            let start_id: Option<u64> = caps.get(1).and_then(|m| m.as_str().parse().ok());
            let end_id: Option<u64> = caps.get(2).and_then(|m| m.as_str().parse().ok());
            assert_eq!(start_id, Some(1200));
            // 断言 end_id 为 None
            assert_eq!(end_id, None);
        } else {
            println!("未匹配到任何内容");
        }

        let text = r#"{"p":"erc-20","op":"mint","tick":"pi","id":"[-2000]","to":"[address]","amt":"1000"}"#;

        if let Some(caps) = re.captures(&text) {
            let start_id: Option<u64> = caps.get(1).and_then(|m| m.as_str().parse().ok());
            let end_id: Option<u64> = caps.get(2).and_then(|m| m.as_str().parse().ok());
            assert_eq!(start_id, None);
            // 断言 end_id 为 None
            assert_eq!(end_id, Some(2000));
        } else {
            println!("未匹配到任何内容");
        }

        let text = r#"{"p":"erc-20","op":"mint","tick":"pi","id":"[1200-2000]","to":"[address]","amt":"1000"}"#;

        if let Some(caps) = re.captures(&text) {
            let start_id: Option<u64> = caps.get(1).and_then(|m| m.as_str().parse().ok());
            let end_id: Option<u64> = caps.get(2).and_then(|m| m.as_str().parse().ok());
            assert_eq!(start_id, Some(1200));
            // 断言 end_id 为 None
            assert_eq!(end_id, Some(2000));
        } else {
            println!("未匹配到任何内容");
        }
    }
}


