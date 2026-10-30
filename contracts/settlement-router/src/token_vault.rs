use crate::errors::SettlementError;
use soroban_sdk::{token::Client as TokenClient, Address, Env};

pub struct TokenVault;

impl TokenVault {
    pub fn transfer_from(
        env: &Env,
        token: &Address,
        from: &Address,
        to: &Address,
        amount: i128,
    ) -> Result<(), SettlementError> {
        if amount <= 0 {
            return Err(SettlementError::ZeroTokenAmount);
        }
        let client = TokenClient::new(env, token);
        let balance = client.balance(from);
        if balance < amount {
            return Err(SettlementError::InsufficientTokenBalance);
        }
        client.transfer_from(&env.current_contract_address(), from, to, &amount);
        Ok(())
    }

    pub fn transfer(
        env: &Env,
        token: &Address,
        to: &Address,
        amount: i128,
    ) -> Result<(), SettlementError> {
        if amount <= 0 {
            return Err(SettlementError::ZeroTokenAmount);
        }
        let client = TokenClient::new(env, token);
        client.transfer(&env.current_contract_address(), to, &amount);
        Ok(())
    }

    pub fn transfer_direct(
        env: &Env,
        token: &Address,
        from: &Address,
        to: &Address,
        amount: i128,
    ) -> Result<(), SettlementError> {
        if amount <= 0 {
            return Err(SettlementError::ZeroTokenAmount);
        }
        let client = TokenClient::new(env, token);
        client.transfer(from, to, &amount);
        Ok(())
    }
}
