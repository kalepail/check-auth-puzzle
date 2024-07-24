#![cfg(test)]

use std::println;
extern crate std;

use ed25519_dalek::{Keypair, Signer};
use puzzle::{Contract as PuzzleContract, Signature};
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

use crate::{Contract as SolverContract, ContractClient as SolverContractClient};

#[test]
fn test() {
    // let env = Env::default();
    let env = Env::from_ledger_snapshot_file("snapshot.json");

    // env.ledger().set_sequence_number(0);

    // let puzzle_id = env.register_contract(None, PuzzleContract);
    let puzzle_id = Address::from_string(&String::from_str(
        &env,
        "CCPYY3EQZQ6SQE2XRHCU5VVH4DCR3ZCNRHIS5ITCSMBK2WOPMN56LEAV",
    ));

    let solver_id = env.register_contract(None, SolverContract);
    // let solver_id = Address::from_string(&String::from_str(
    //     &env,
    //     "CD37S3GCDMYYYMMQFL4EG555OQLVR6ZSY5ZWABTMYH2Y7WRBZJCHVFST",
    // ));
    let solver_client = SolverContractClient::new(&env, &solver_id);

    // let sac = env.register_stellar_asset_contract(Address::generate(&env));
    let sac = Address::from_string(&String::from_str(
        &env,
        "CDGOXJBEKI3MQDB3J477NN3HAQBDCNK5YYB2ZKAG24US53RXW44QIF6Z",
    ));
    let token_client = token::Client::new(&env, &sac);

    // let sac_client = token::StellarAssetClient::new(&env, &sac);
    // sac_client.mock_all_auths().mint(&contract_id, &i128::MAX);

    // let keypair = Keypair::generate(&mut thread_rng());
    let pubkey = Address::from_string(&String::from_str(
        &env,
        "GCQXHDLSMF6YR53VNE6JXEBT3C53THISP2U2YDYESQG5BEBVBRNU4HZH",
    ));
    let keypair = Keypair::from_bytes(&[
        88, 206, 67, 128, 240, 45, 168, 148, 191, 111, 180, 111, 104, 83, 214, 113, 78,
        27, 55, 86, 200, 247, 164, 163, 76, 236, 24, 208, 115, 40, 231, 255, 161, 115, 141,
        114, 97, 125, 136, 247, 117, 105, 60, 155, 144, 51, 216, 187, 185, 157, 18, 126,
        169, 172, 15, 4, 148, 13, 208, 144, 53, 12, 91, 78
      ])
    .unwrap();

    println!("{}", token_client.balance(&puzzle_id));
    println!("{}", token_client.balance(&pubkey));

    let mut tax = 0;

    env.as_contract(&env.register_contract(None, SolverContract), || {
        let mut seed = [0u8; 32];

        let balance = token_client.balance(&puzzle_id);

        seed[..16].swap_with_slice(&mut balance.to_be_bytes());

        env.prng().seed(Bytes::from_array(&env, &seed));

        tax = env.prng().gen::<u64>() as u32;
    });

    println!("\n{}\n", tax);

    let nonce = 0;
    let signature_expiration_ledger = env.ledger().sequence();
    let root_invocation = SorobanAuthorizedInvocation {
        function: SorobanAuthorizedFunction::ContractFn(InvokeContractArgs {
            contract_address: solver_id.clone().try_into().unwrap(),
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

    solver_client
        .set_auths(&[SorobanAuthorizationEntry {
            credentials: SorobanCredentials::Address(SorobanAddressCredentials {
                address: puzzle_id.clone().try_into().unwrap(),
                nonce,
                signature_expiration_ledger,
                signature: Signature {
                    address: address.clone(),
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
        .call(&puzzle_id, &sac);

    println!("{}", token_client.balance(&puzzle_id));
    println!("{}", token_client.balance(&pubkey));
}
