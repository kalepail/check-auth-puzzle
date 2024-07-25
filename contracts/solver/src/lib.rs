#![no_std]

use soroban_sdk::{contract, contractimpl, token, vec, Address, Bytes, Env, IntoVal};

mod test;

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn call(env: Env, 
        // address: Address, 
        puzzle: Address, sac: Address) {
        // let token = token::TokenClient::new(&env, &sac);

        // token.transfer(&address, &puzzle, &10_000_000);

        puzzle.require_auth_for_args(vec![&env, sac.into_val(&env)]);
    }
}
