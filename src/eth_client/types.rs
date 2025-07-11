use std::collections::{HashMap, HashSet};

use alloy::primitives::{Address, U256, address};

pub const USDC: Address = address!("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
pub const WETH: Address = address!("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
pub const WBTC: Address = address!("0x2260fac5e5542a773aa44fbcfedf7c193bc2c599");

pub const KNOWN_TOKENS: &[Address] = &[USDC, WETH, WBTC];

pub struct ParsedData {
    pub block_id: u64,
    /// A map of which account interacted with which contracts.
    pub interactions: HashMap<Address, HashSet<Address>>,
}

pub struct Balance {
    pub account: Address,
    pub balance: U256,
    pub token: Address,
    pub block_id: u64,
}

impl Balance {
    pub fn into(self) -> crate::types::Balance {
        crate::types::Balance {
            account: self.account.into(),
            token: self.token.into(),
            balance: self.balance.to_be_bytes(),
            block_id: self.block_id,
        }
    }
}
