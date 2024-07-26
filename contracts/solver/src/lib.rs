#![no_std]

use soroban_sdk::{contract, contractimpl, vec, Address, Env, IntoVal};

mod test;

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn call(env: Env, puzzle: Address, sac: Address) {
        puzzle.require_auth_for_args(vec![&env, sac.into_val(&env)]);
    }
}
