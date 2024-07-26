import { Buffer } from "buffer";
import { Address } from '@stellar/stellar-sdk';
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  Result,
  Spec as ContractSpec,
} from '@stellar/stellar-sdk/contract';
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Typepoint,
  Duration,
} from '@stellar/stellar-sdk/contract';
export * from '@stellar/stellar-sdk'
export * as contract from '@stellar/stellar-sdk/contract'
export * as rpc from '@stellar/stellar-sdk/rpc'

if (typeof window !== 'undefined') {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}


export const networks = {
  testnet: {
    networkPassphrase: "Test SDF Network ; September 2015",
    contractId: "CAQCCHB3DXHNCBM63KZKDDB433X633HEKOUZCIEH32E74KPZQFVKMUHH",
  }
} as const


export interface Signature {
  address: string;
  signature: Buffer;
}

export type DataKey = {tag: "In", values: void} | {tag: "Out", values: void};

export const Errors = {
  1: {message:"TooBadSoSad"}
}

export interface Client {
  /**
   * Construct and simulate a setup transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  setup: ({sac_in_address, sac_out_address}: {sac_in_address: string, sac_out_address: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<null>>

}
export class Client extends ContractClient {
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAQAAAAAAAAAAAAAACVNpZ25hdHVyZQAAAAAAAAIAAAAAAAAAB2FkZHJlc3MAAAAAEwAAAAAAAAAJc2lnbmF0dXJlAAAAAAAD7gAAAEA=",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAAAgAAAAAAAAAAAAAAAkluAAAAAAAAAAAAAAAAAANPdXQA",
        "AAAABAAAAAAAAAAAAAAABUVycm9yAAAAAAAAAQAAAAAAAAALVG9vQmFkU29TYWQAAAAAAQ==",
        "AAAAAAAAAAAAAAAFc2V0dXAAAAAAAAACAAAAAAAAAA5zYWNfaW5fYWRkcmVzcwAAAAAAEwAAAAAAAAAPc2FjX291dF9hZGRyZXNzAAAAABMAAAAA",
        "AAAAAAAAAAAAAAAMX19jaGVja19hdXRoAAAAAwAAAAAAAAARc2lnbmF0dXJlX3BheWxvYWQAAAAAAAPuAAAAIAAAAAAAAAAJc2lnbmF0dXJlAAAAAAAH0AAAAAlTaWduYXR1cmUAAAAAAAAAAAAADl9hdXRoX2NvbnRleHRzAAAAAAPqAAAH0AAAAAdDb250ZXh0AAAAAAEAAAPpAAAD7QAAAAAAAAAD" ]),
      options
    )
  }
  public readonly fromJSON = {
    setup: this.txFromJSON<null>
  }
}