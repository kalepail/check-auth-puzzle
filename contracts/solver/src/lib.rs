#![no_std]

use soroban_sdk::{contract, contractimpl, token, vec, Address, Bytes, Env, IntoVal};

mod test;

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn call(env: Env, puzzle: Address, sac: Address) {
        let mut seed = [0u8; 32];
        let token = token::Client::new(&env, &sac);
        let balance = token.balance(&puzzle);

        seed[..16].swap_with_slice(&mut balance.to_be_bytes());
        env.prng().seed(Bytes::from_array(&env, &seed));

        let tax = env.prng().gen::<u64>() as u32;

        puzzle.require_auth_for_args(vec![&env, sac.into_val(&env), tax.into_val(&env)]);
    }
}
