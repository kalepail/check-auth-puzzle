#![cfg(test)]

use std::println;
extern crate std;

use ed25519_dalek::{Keypair, Signer};
use puzzle::{Contract as PuzzleContract, Error, Signature};
use rand::thread_rng;
use soroban_sdk::{
    auth::{self, Context, ContractContext}, contracttype, symbol_short, testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke}, token, vec, xdr::{
        BytesM, HashIdPreimage, HashIdPreimageSorobanAuthorization, Int128Parts, InvokeContractArgs, Limits, ScAddress, ScBytes, ScVal, ScVec, SorobanAddressCredentials, SorobanAuthorizationEntry, SorobanAuthorizedFunction, SorobanAuthorizedInvocation, SorobanCredentials, VecM, WriteXdr
    }, Address, Bytes, BytesN, Env, IntoVal, String
};
use stellar_strkey::{ed25519, Contract, Strkey};

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

    let puzzle_id = env.register_contract(None, PuzzleContract);

    let solver_id = env.register_contract(None, SolverContract);
    let solver_client = SolverContractClient::new(&env, &solver_id);

    // let sac = env.register_stellar_asset_contract(Address::generate(&env));
    let sac = Address::from_string(&String::from_str(
        &env,
        "CDGOXJBEKI3MQDB3J477NN3HAQBDCNK5YYB2ZKAG24US53RXW44QIF6Z",
    ));
    // let sac_client = token::StellarAssetClient::new(&env, &sac);
    // let token_client = token::Client::new(&env, &sac);

    // let pubkey = Address::from_string(&String::from_str(
    //     &env,
    //     "GCQXHDLSMF6YR53VNE6JXEBT3C53THISP2U2YDYESQG5BEBVBRNU4HZH",
    // ));
    let keypair = Keypair::from_bytes(&[
        88, 206, 67, 128, 240, 45, 168, 148, 191, 111, 180, 111, 104, 83, 214, 113, 78,
        27, 55, 86, 200, 247, 164, 163, 76, 236, 24, 208, 115, 40, 231, 255, 161, 115, 141,
        114, 97, 125, 136, 247, 117, 105, 60, 155, 144, 51, 216, 187, 185, 157, 18, 126,
        169, 172, 15, 4, 148, 13, 208, 144, 53, 12, 91, 78
      ])
    .unwrap();

    let address = Strkey::PublicKeyEd25519(ed25519::PublicKey(keypair.public.to_bytes()));
    let address = Bytes::from_slice(&env, address.to_string().as_bytes());
    let address = Address::from_string_bytes(&address);

    let signature_expiration_ledger = env.ledger().sequence();

    let invocation_1 = SorobanAuthorizedInvocation {
        function: SorobanAuthorizedFunction::ContractFn(InvokeContractArgs {
            contract_address: solver_id.clone().try_into().unwrap(),
            function_name: "call".try_into().unwrap(),
            args: std::vec![
                ScVal::Address(ScAddress::try_from(sac.clone()).unwrap()),
            ]
            .try_into()
            .unwrap(),
        }),
        sub_invocations: VecM::default()
    };

    let payload_1_preimage = HashIdPreimage::SorobanAuthorization(HashIdPreimageSorobanAuthorization {
        network_id: env.ledger().network_id().to_array().into(),
        nonce: 1111,
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
            contract_address: sac.clone().try_into().unwrap(),
            function_name: "transfer".try_into().unwrap(),
            args: std::vec![
                ScVal::Address(ScAddress::try_from(address.clone()).unwrap()),
                ScVal::Address(ScAddress::try_from(puzzle_id.clone()).unwrap()),
                ScVal::I128(Int128Parts{ hi: 0, lo: 10_000_000 })
            ]
            .try_into()
            .unwrap(),
        }),
        sub_invocations: VecM::default(),
    };

    let payload_2_preimage = HashIdPreimage::SorobanAuthorization(HashIdPreimageSorobanAuthorization {
        network_id: env.ledger().network_id().to_array().into(),
        nonce: 2222,
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

    solver_client
        .set_auths(&[
            SorobanAuthorizationEntry {
                credentials: SorobanCredentials::Address(SorobanAddressCredentials {
                    address: puzzle_id.clone().try_into().unwrap(),
                    nonce: 1111,
                    signature_expiration_ledger,
                    signature: Signature {
                        address: address.clone(),
                        signature: BytesN::from_array(
                            &env,
                            &keypair.sign(payload_1_hash.to_array().as_slice()).to_bytes(),
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
                    nonce: 2222,
                    signature: std::vec![
                        AccountEd25519Signature {
                            public_key: BytesN::from_array(&env, &keypair.public.to_bytes()),
                            signature: BytesN::from_array(&env, &keypair.sign(&payload_2_hash.to_array()).to_bytes()),
                        }
                    ].try_into().unwrap(),
                    signature_expiration_ledger,
                }),
                root_invocation: invocation_2,
            }
        ])
        // .mock_all_auths()
        .call(
            // &address, 
            &puzzle_id, &sac);

    env.auths().iter().for_each(|auth| {
        println!("{:?}", auth);
    });
}

// SorobanAuthorizationEntry {
//     credentials: SorobanCredentials::Address(SorobanAddressCredentials {
//         address: solver_id.clone().try_into().unwrap(),
//         nonce: 1111,
//         signature: ScVal::Void,
//         signature_expiration_ledger,
//     }),
//     root_invocation: invocation_1,
// },
// SorobanAuthorizationEntry {
//     credentials: SorobanCredentials::Address(SorobanAddressCredentials {
//         address: puzzle_id.clone().try_into().unwrap(),
//         nonce: 2222,
//         signature: ScVal::Void,
//         signature_expiration_ledger,
//     }),
//     root_invocation: invocation_2,
// },
// SorobanAuthorizationEntry {
//     credentials: SorobanCredentials::Address(SorobanAddressCredentials {
//         address: address.clone().try_into().unwrap(),
//         nonce: 3333,
//         signature: Signature {
//             address: address.clone(),
//             signature: BytesN::from_array(
//                 &env,
//                 &keypair.sign(payload_3_hash.to_array().as_slice()).to_bytes(),
//             ),
//         }
//         .try_into()
//         .unwrap(),
//         signature_expiration_ledger,
//     }),
//     root_invocation: invocation_3,
// }


// SorobanAuthorizationEntry {
//     credentials: SorobanCredentials::Address(SorobanAddressCredentials {
//         address: puzzle_id.clone().try_into().unwrap(),
//         nonce,
//         signature_expiration_ledger,
//         signature: Signature {
//             address: address.clone(),
//             signature: BytesN::from_array(
//                 &env,
//                 &keypair.sign(payload.to_array().as_slice()).to_bytes(),
//             ),
//         }
//         .try_into()
//         .unwrap(),
//     }),
//     root_invocation: root_invocation.clone(),
// },


// SorobanAuthorizationEntry {
//     credentials: SorobanCredentials::Address(SorobanAddressCredentials {
//         address: sac.clone().try_into().unwrap(),
//         nonce,
//         signature_expiration_ledger,
//         signature: keypair.sign(dayload.to_array().as_slice()).to_bytes()
//         .try_into()
//         .unwrap(),
//     }),
//     root_invocation: toot_invocation.clone(),
// },


// let invocation_2 = SorobanAuthorizedInvocation {
//     function: SorobanAuthorizedFunction::ContractFn(InvokeContractArgs {
//         contract_address: puzzle_id.clone().try_into().unwrap(),
//         function_name: "__check_auth".try_into().unwrap(),
//         args: std::vec![
//             ScVal::Bytes(ScBytes(BytesM::try_from(payload_1_hash.to_array()).unwrap())),
//         ]
//         .try_into()
//         .unwrap(),
//     }),
//     sub_invocations: VecM::default(),
// };

// let payload_2_preimage = HashIdPreimage::SorobanAuthorization(HashIdPreimageSorobanAuthorization {
//     network_id: env.ledger().network_id().to_array().into(),
//     nonce: 2222,
//     signature_expiration_ledger,
//     invocation: invocation_2.clone(),
// });

// let payload_2_xdr = payload_2_preimage
//     .to_xdr(Limits {
//         depth: u32::MAX,
//         len: usize::MAX,
//     })
//     .unwrap();

// let mut payload_2 = Bytes::new(&env);

// for byte in payload_2_xdr.iter() {
//     payload_2.push_back(*byte);
// }

// let payload_2_hash = env.crypto().sha256(&payload_2);

// let invocation_3 = SorobanAuthorizedInvocation {
//     function: SorobanAuthorizedFunction::ContractFn(InvokeContractArgs {
//         contract_address: puzzle_id.clone().try_into().unwrap(),
//         function_name: "__check_auth".try_into().unwrap(),
//         args: std::vec![
//             ScVal::Bytes(ScBytes(BytesM::try_from(payload_2_hash.to_array()).unwrap())),
//         ]
//         .try_into()
//         .unwrap(),
//     }),
//     sub_invocations: VecM::default(),
// };

// let payload_3_preimage = HashIdPreimage::SorobanAuthorization(HashIdPreimageSorobanAuthorization {
//     network_id: env.ledger().network_id().to_array().into(),
//     nonce: 3333,
//     signature_expiration_ledger,
//     invocation: invocation_3.clone(),
// });

// let payload_3_xdr = payload_3_preimage
//     .to_xdr(Limits {
//         depth: u32::MAX,
//         len: usize::MAX,
//     })
//     .unwrap();

// let mut payload_3 = Bytes::new(&env);

// for byte in payload_3_xdr.iter() {
//     payload_3.push_back(*byte);
// }

// let payload_3_hash = env.crypto().sha256(&payload_3);
////

////
// let root_invocation = 

// let payload = HashIdPreimage::SorobanAuthorization(HashIdPreimageSorobanAuthorization {
//     network_id: env.ledger().network_id().to_array().into(),
//     nonce,
//     signature_expiration_ledger,
//     invocation: root_invocation.clone(),
// });

// let payload_xdr = payload
//     .to_xdr(Limits {
//         depth: u32::MAX,
//         len: usize::MAX,
//     })
//     .unwrap();

// let mut payload = Bytes::new(&env);

// for byte in payload_xdr.iter() {
//     payload.push_back(*byte);
// }

// let payload = env.crypto().sha256(&payload);

// sac_client.mock_all_auths().mint(&address, &i128::MAX);
////

// println!("{:?}", token_client.balance(&address));

// let res = env.try_invoke_contract_check_auth::<Error>(
//     &puzzle_id, 
//     &env.crypto().sha256(&payload).try_into().unwrap(),
//     Signature {
//         address: address.clone(),
//         signature: BytesN::from_array(
//             &env,
//             &keypair.sign(env.crypto().sha256(&payload).to_array().as_slice()).to_bytes(),
//         ),
//     }.into_val(&env),
//     &vec![&env, Context::Contract(ContractContext {
//         contract: solver_id,
//         fn_name: symbol_short!["call"],
//         args: vec![&env, sac.to_val()],
//     })]
// );

// println!("{:?}", res);

// println!("{:?}", token_client.balance(&address));