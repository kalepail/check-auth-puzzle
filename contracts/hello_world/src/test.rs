#![cfg(test)]

use std::println;
extern crate std;

use ed25519_dalek::{Keypair, Signer};
use rand::thread_rng;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token,
    xdr::{
        HashIdPreimage, HashIdPreimageSorobanAuthorization, InvokeContractArgs, Limits, ScAddress,
        ScVal, SorobanAddressCredentials, SorobanAuthorizationEntry, SorobanAuthorizedFunction,
        SorobanAuthorizedInvocation, SorobanCredentials, VecM, WriteXdr,
    },
    Address, Bytes, BytesN, Env, String,
};
use stellar_strkey::{ed25519, Strkey};

use crate::{Contract, ContractClient, Signature};

#[test]
fn test() {
    let env = Env::from_ledger_snapshot_file("snapshot.json"); // Env::default();

    // env.ledger().set_sequence_number(0);

    let seed_contract_id = env.register_contract(None, Contract);
    // let seed_contract_id = Address::from_string(&String::from_str(
    //     &env,
    //     "CADLLO22G7IXC4KYDRVSXCW5M2DEZYY5KT3AZT4AFQOBH527VTB76XIE",
    // ));
    
    // let contract_id = env.register_contract(None, Contract);
    // let client = ContractClient::new(&env, &contract_id);
    let contract_id = Address::from_string(&String::from_str(
        &env,
        "CAKX2ZMAKMID6PEDSUHBMU4NAHWUIEFXTXHESCSN7IRVG5E4QKAWSGLU",
    ));
    let client = ContractClient::new(&env, &contract_id);

    // let sac = env.register_stellar_asset_contract(Address::generate(&env));
    // let sac_client = token::StellarAssetClient::new(&env, &sac);
    // let token_client = token::Client::new(&env, &sac);
    let sac = Address::from_string(&String::from_str(
        &env,
        "CALCROAXSHD3HWE3O2EBJIGGWFMXD24725XIQL5P3IZHA6DE3ETO3NU2",
    ));
    let token_client = token::Client::new(&env, &sac);

    // let keypair = Keypair::generate(&mut thread_rng());
    let keypair = Keypair::from_bytes(&[
        219, 45, 139, 152, 135, 66, 48, 220, 57, 239, 152, 64, 255, 188, 170, 241, 190, 234, 178,
        177, 147, 150, 219, 118, 232, 145, 75, 145, 27, 97, 3, 59, 231, 181, 37, 137, 68, 44, 55,
        102, 196, 136, 182, 174, 202, 70, 129, 176, 5, 207, 85, 149, 1, 7, 235, 4, 48, 213, 252,
        13, 242, 133, 168, 36,
    ])
    .unwrap();

    // sac_client.mock_all_auths().mint(&contract_id, &i128::MAX);

    env.as_contract(&seed_contract_id.clone(), || {
        let mut seed = [0u8; 32];

        let balance = token_client.balance(&contract_id);

        seed[..16].swap_with_slice(&mut balance.to_be_bytes());

        env.prng().seed(Bytes::from_array(&env, &seed));

        let tax = env.prng().gen::<u64>() as u32;

        let nonce = 0;
        let signature_expiration_ledger = env.ledger().sequence() + 1;
        let root_invocation = SorobanAuthorizedInvocation {
            function: SorobanAuthorizedFunction::ContractFn(InvokeContractArgs {
                contract_address: contract_id.clone().try_into().unwrap(),
                function_name: "call".try_into().unwrap(),
                args: std::vec![
                    ScVal::Address(ScAddress::try_from(sac.clone()).unwrap()),
                    ScVal::U32(tax)
                ]
                .try_into()
                .unwrap(),
            }),
            sub_invocations: VecM::default(),
        };

        let payload = HashIdPreimage::SorobanAuthorization(HashIdPreimageSorobanAuthorization {
            network_id: env.ledger().network_id().to_array().into(),
            nonce,
            signature_expiration_ledger,
            invocation: root_invocation.clone(),
        });

        let payload_xdr = payload
            .to_xdr(Limits {
                depth: u32::MAX,
                len: usize::MAX,
            })
            .unwrap();

        let mut payload = Bytes::new(&env);

        for byte in payload_xdr.iter() {
            payload.push_back(*byte);
        }

        let payload = env.crypto().sha256(&payload);

        let address = Strkey::PublicKeyEd25519(ed25519::PublicKey(keypair.public.to_bytes()));
        let address = Bytes::from_slice(&env, address.to_string().as_bytes());
        let address = Address::from_string_bytes(&address);

        client
            .set_auths(&[SorobanAuthorizationEntry {
                credentials: SorobanCredentials::Address(SorobanAddressCredentials {
                    address: contract_id.clone().try_into().unwrap(),
                    nonce,
                    signature_expiration_ledger,
                    signature: Signature {
                        address: address.clone(),
                        public_key: BytesN::from_array(&env, &keypair.public.to_bytes()),
                        signature: BytesN::from_array(
                            &env,
                            &keypair.sign(payload.to_array().as_slice()).to_bytes(),
                        ),
                    }
                    .try_into()
                    .unwrap(),
                }),
                root_invocation,
            }])
            .call(&sac);

        // println!("{}", token_client.balance(&address));
    });
}
