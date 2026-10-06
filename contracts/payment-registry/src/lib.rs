#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol};

const COUNTER_KEY: Symbol = symbol_short!("counter");

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentRecord {
    pub id: u64,
    pub sender: Address,
    pub recipient: Address,
    pub amount: i128,
    pub memo: String,
    pub timestamp: u64,
}

#[contract]
pub struct PaymentRegistry;

#[contractimpl]
impl PaymentRegistry {
    /// Level 1 Foundation check: returns registry status and protocol version.
    pub fn status(_env: Env) -> Symbol {
        symbol_short!("ready")
    }

    /// Read the total number of registered payments in this registry.
    pub fn get_payment_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&COUNTER_KEY)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_registry_scaffold_status() {
        let env = Env::default();
        let contract_id = env.register(PaymentRegistry, ());
        let client = PaymentRegistryClient::new(&env, &contract_id);

        assert_eq!(client.status(), symbol_short!("ready"));
        assert_eq!(client.get_payment_count(), 0);
    }
}
