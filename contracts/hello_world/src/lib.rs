#![no_std]

use soroban_sdk::{
    auth::{Context, CustomAccountInterface},
    contract, contracterror, contractimpl, contracttype,
    crypto::Hash,
    token, vec,
    xdr::ToXdr,
    Address, Bytes, BytesN, Env, IntoVal, TryIntoVal, Vec,
};

mod test;

#[contract]
pub struct Contract;

// TODO remove this function and create a new "solve" contract that employs this and calls the __check_auth from this contract
// When we do that we'll also need to implement the seed and prng logic into the __check_auth function here

#[contractimpl]
impl Contract {
    pub fn call(env: Env, sac: Address) {
        let mut seed = [0u8; 32];

        let token = token::Client::new(&env, &sac);
        let balance = token.balance(&env.current_contract_address());

        seed[..16].swap_with_slice(&mut balance.to_be_bytes());

        env.prng().seed(Bytes::from_array(&env, &seed));

        let tax = env.prng().gen::<u64>() as u32;

        env.current_contract_address().require_auth_for_args(vec![
            &env,
            sac.into_val(&env),
            tax.into_val(&env),
        ]);
    }
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Signature {
    pub address: Address,
    pub public_key: BytesN<32>,
    pub signature: BytesN<64>,
}

#[contracterror]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u32)]
pub enum Error {
    TooBadSoSad = 1,
}

#[contractimpl]
impl CustomAccountInterface for Contract {
    type Error = Error;
    type Signature = Signature;

    #[allow(non_snake_case)]
    fn __check_auth(
        env: Env,
        signature_payload: Hash<32>,
        signature: Signature,
        auth_contexts: Vec<Context>,
    ) -> Result<(), Error> {
        env.crypto().ed25519_verify(
            &signature.public_key,
            &signature_payload.into(),
            &signature.signature,
        );

        for context in auth_contexts.iter() {
            match context {
                Context::Contract(c) => {
                    let sac: Address = c.args.get(0).unwrap().try_into_val(&env).unwrap();
                    let token = token::Client::new(&env, &sac);
                    let amount: u32 = c.args.get(1).unwrap().try_into_val(&env).unwrap();
                    let to_bytes = signature.address.clone().to_xdr(&env);

                    if to_bytes.slice(to_bytes.len() - 32..)
                        != Bytes::from_array(&env, &signature.public_key.to_array())
                    {
                        return Err(Error::TooBadSoSad);
                    }

                    token.transfer(&c.contract, &signature.address, &(amount as i128));
                }
                _ => {}
            }
        }

        Ok(())
    }
}
