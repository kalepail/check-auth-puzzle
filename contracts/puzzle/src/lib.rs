#![no_std]

use soroban_sdk::{
    auth::{Context, CustomAccountInterface},
    contract, contracterror, contractimpl, contracttype,
    crypto::Hash,
    panic_with_error, token,
    xdr::ToXdr,
    Address, BytesN, Env, Vec,
};

#[contract]
pub struct Contract;

#[contracttype]
#[derive(Clone, Debug)]
pub struct Signature {
    pub address: Address,
    pub signature: BytesN<64>,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    In,
    Out,
}

#[contracterror]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u32)]
pub enum Error {
    TooBadSoSad = 1,
}

#[contractimpl]
impl Contract {
    pub fn setup(env: Env, sac_in_address: Address, sac_out_address: Address) {
        env.storage().temporary().set(&DataKey::In, &sac_in_address);
        env.storage()
            .temporary()
            .set(&DataKey::Out, &sac_out_address);
    }
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
        _auth_contexts: Vec<Context>,
    ) -> Result<(), Error> {
        let address_bytes = signature.address.clone().to_xdr(&env);
        let address_bytes = address_bytes.slice(address_bytes.len() - 32..);

        let mut slice = [0u8; 32];
        address_bytes.copy_into_slice(&mut slice);

        let public_key = BytesN::from_array(&env, &slice);

        env.crypto()
            .ed25519_verify(&public_key, &signature_payload.into(), &signature.signature);

        let sac_in_client = token::TokenClient::new(
            &env,
            &env.storage()
                .temporary()
                .get::<DataKey, Address>(&DataKey::In)
                .unwrap_or_else(|| panic_with_error!(&env, Error::TooBadSoSad)),
        );
        let sac_out_client = token::TokenClient::new(
            &env,
            &env.storage()
                .temporary()
                .get::<DataKey, Address>(&DataKey::Out)
                .unwrap_or_else(|| panic_with_error!(&env, Error::TooBadSoSad)),
        );

        sac_in_client.transfer(
            &signature.address,
            &env.current_contract_address(),
            &10_000_000,
        );
        sac_out_client.transfer(
            &env.current_contract_address(),
            &signature.address,
            &10_000_000,
        );

        Ok(())
    }
}
