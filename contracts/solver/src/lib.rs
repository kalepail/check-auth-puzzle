#![no_std]

use soroban_sdk::{contract, contractimpl, vec, Address, Env};

mod test;

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn call(env: Env, puzzle_address: Address) {
        puzzle_address.require_auth_for_args(vec![&env]);
    }
}
