#![no_std]

use soroban_sdk::{
    auth::{Context, CustomAccountInterface},
    contract, contracterror, contractimpl, contracttype,
    crypto::Hash,
    token,
    xdr::ToXdr,
    Address, Bytes, BytesN, Env, TryIntoVal, Vec,
};

#[contract]
pub struct Contract;

#[contracttype]
#[derive(Clone, Debug)]
pub struct Signature {
    pub address: Address,
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
        let address_bytes = signature.address.clone().to_xdr(&env);
        let address_bytes = address_bytes.slice(address_bytes.len() - 32..);

        let mut slice = [0u8; 32];
        address_bytes.copy_into_slice(&mut slice);

        let public_key = BytesN::from_array(&env, &slice);

        env.crypto()
            .ed25519_verify(&public_key, &signature_payload.into(), &signature.signature);

        for context in auth_contexts.iter() {
            match context {
                Context::Contract(c) => {
                    let mut seed = [0u8; 32];
                    let sac: Address = c.args.get(0).unwrap().try_into_val(&env).unwrap();
                    let amount: u32 = c.args.get(1).unwrap().try_into_val(&env).unwrap();
                    let token = token::TokenClient::new(&env, &sac);
                    let balance = token.balance(&env.current_contract_address());

                    seed[..16].swap_with_slice(&mut balance.to_be_bytes());
                    env.prng().seed(Bytes::from_array(&env, &seed));

                    if env.prng().gen::<u64>() as u32 != amount {
                        return Err(Error::TooBadSoSad);
                    }

                    token.transfer(
                        &env.current_contract_address(),
                        &signature.address,
                        &10_000_000
                    );
                }
                _ => {}
            }
        }

        Ok(())
    }
}