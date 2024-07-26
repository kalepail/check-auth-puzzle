#![cfg(test)]

use std::println;
extern crate std;

use ed25519_dalek::{Keypair, Signer};
use puzzle::{Contract as PuzzleContract, ContractClient as PuzzleContractClient, Signature};
use soroban_sdk::{
    contracttype, token,
    xdr::{
        HashIdPreimage, HashIdPreimageSorobanAuthorization, Int128Parts, InvokeContractArgs,
        Limits, ScAddress, ScVal, SorobanAddressCredentials, SorobanAuthorizationEntry,
        SorobanAuthorizedFunction, SorobanAuthorizedInvocation, SorobanCredentials, VecM, WriteXdr,
    },
    Address, Bytes, BytesN, Env, String,
};
use stellar_strkey::{ed25519, Strkey};

use crate::{Contract as SolverContract, ContractClient as SolverContractClient};

#[contracttype]
struct AccountEd25519Signature {
    public_key: BytesN<32>,
    signature: BytesN<64>,
}

#[test]
fn test() {
    // let env = Env::default();
    let env = Env::from_ledger_snapshot_file("snapshot.json");

    // env.ledger().set_sequence_number(0);

    let puzzle_address = env.register_contract(None, PuzzleContract);
    let puzzle_client = PuzzleContractClient::new(&env, &puzzle_address);

    let solver_address = env.register_contract(None, SolverContract);
    let solver_client = SolverContractClient::new(&env, &solver_address);

    let sac_in_address = Address::from_string(&String::from_str(
        &env,
        "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC",
    ));
    // let sac_in_admin = token::StellarAssetClient::new(&env, &sac_in_address);
    // let sac_in_client = token::Client::new(&env, &sac_in_address);

    // let puzzle_sac_address = env.register_stellar_asset_contract(Address::generate(&env));
    let sac_out_address = Address::from_string(&String::from_str(
        &env,
        "CDGOXJBEKI3MQDB3J477NN3HAQBDCNK5YYB2ZKAG24US53RXW44QIF6Z",
    ));
    let sac_out_admin = token::StellarAssetClient::new(&env, &sac_out_address);
    // let sac_out_client = token::Client::new(&env, &sac_out_address);

    sac_out_admin
        .mock_all_auths()
        .mint(&puzzle_address, &10_000_000);

    // let pubkey = Address::from_string(&String::from_str(
    //     &env,
    //     "GCQXHDLSMF6YR53VNE6JXEBT3C53THISP2U2YDYESQG5BEBVBRNU4HZH",
    // ));
    let keypair = Keypair::from_bytes(&[
        88, 206, 67, 128, 240, 45, 168, 148, 191, 111, 180, 111, 104, 83, 214, 113, 78, 27, 55, 86,
        200, 247, 164, 163, 76, 236, 24, 208, 115, 40, 231, 255, 161, 115, 141, 114, 97, 125, 136,
        247, 117, 105, 60, 155, 144, 51, 216, 187, 185, 157, 18, 126, 169, 172, 15, 4, 148, 13,
        208, 144, 53, 12, 91, 78,
    ])
    .unwrap();

    let address = Strkey::PublicKeyEd25519(ed25519::PublicKey(keypair.public.to_bytes()));
    let address = Bytes::from_slice(&env, address.to_string().as_bytes());
    let address = Address::from_string_bytes(&address);

    let signature_expiration_ledger = env.ledger().sequence();

    let invocation_1 = SorobanAuthorizedInvocation {
        function: SorobanAuthorizedFunction::ContractFn(InvokeContractArgs {
            contract_address: solver_address.clone().try_into().unwrap(),
            function_name: "call".try_into().unwrap(),
            args: VecM::default(),
        }),
        sub_invocations: VecM::default(),
    };

    let payload_1_preimage =
        HashIdPreimage::SorobanAuthorization(HashIdPreimageSorobanAuthorization {
            network_id: env.ledger().network_id().to_array().into(),
            nonce: 0,
            signature_expiration_ledger,
            invocation: invocation_1.clone(),
        });

    let payload_1_xdr = payload_1_preimage
        .to_xdr(Limits {
            depth: u32::MAX,
            len: usize::MAX,
        })
        .unwrap();

    let mut payload_1 = Bytes::new(&env);

    for byte in payload_1_xdr.iter() {
        payload_1.push_back(*byte);
    }

    let payload_1_hash = env.crypto().sha256(&payload_1);

    let invocation_2 = SorobanAuthorizedInvocation {
        function: SorobanAuthorizedFunction::ContractFn(InvokeContractArgs {
            contract_address: sac_in_address.clone().try_into().unwrap(),
            function_name: "transfer".try_into().unwrap(),
            args: std::vec![
                ScVal::Address(ScAddress::try_from(address.clone()).unwrap()),
                ScVal::Address(ScAddress::try_from(puzzle_address.clone()).unwrap()),
                ScVal::I128(Int128Parts {
                    hi: 0,
                    lo: 10_000_000
                })
            ]
            .try_into()
            .unwrap(),
        }),
        sub_invocations: VecM::default(),
    };

    let payload_2_preimage =
        HashIdPreimage::SorobanAuthorization(HashIdPreimageSorobanAuthorization {
            network_id: env.ledger().network_id().to_array().into(),
            nonce: 0,
            signature_expiration_ledger,
            invocation: invocation_2.clone(),
        });

    let payload_2_xdr = payload_2_preimage
        .to_xdr(Limits {
            depth: u32::MAX,
            len: usize::MAX,
        })
        .unwrap();

    let mut payload_2 = Bytes::new(&env);

    for byte in payload_2_xdr.iter() {
        payload_2.push_back(*byte);
    }

    let payload_2_hash = env.crypto().sha256(&payload_2);

    puzzle_client.setup(&sac_in_address, &sac_out_address);

    solver_client
        .set_auths(&[
            SorobanAuthorizationEntry {
                credentials: SorobanCredentials::Address(SorobanAddressCredentials {
                    address: puzzle_address.clone().try_into().unwrap(),
                    nonce: 0,
                    signature_expiration_ledger,
                    signature: Signature {
                        address: address.clone(),
                        signature: BytesN::from_array(
                            &env,
                            &keypair
                                .sign(payload_1_hash.to_array().as_slice())
                                .to_bytes(),
                        ),
                    }
                    .try_into()
                    .unwrap(),
                }),
                root_invocation: invocation_1,
            },
            SorobanAuthorizationEntry {
                credentials: SorobanCredentials::Address(SorobanAddressCredentials {
                    address: address.clone().try_into().unwrap(),
                    nonce: 0,
                    signature: std::vec![AccountEd25519Signature {
                        public_key: BytesN::from_array(&env, &keypair.public.to_bytes()),
                        signature: BytesN::from_array(
                            &env,
                            &keypair.sign(&payload_2_hash.to_array()).to_bytes()
                        ),
                    }]
                    .try_into()
                    .unwrap(),
                    signature_expiration_ledger,
                }),
                root_invocation: invocation_2,
            },
        ])
        // .mock_all_auths() // TODO file an issue that this misses internal auth inside __check_auth
        .call(&puzzle_address);

    env.auths().iter().for_each(|auth| {
        println!("{:?}", auth);
    });
}
