import { SorobanRpc, Networks, Keypair, xdr, hash, Address, Operation, TransactionBuilder, Transaction } from '@stellar/stellar-sdk'
import { basicNodeSigner, DEFAULT_TIMEOUT } from '@stellar/stellar-sdk/contract';
import { Client, networks } from 'puzzle-sdk'

// Issuer
// PUZZLE
// SCEVQ4LE2HI7VGJKVJPEZVUALIDIGXWVGVWG5RC22KQRFZIMHYZX7QR2
// GBT7SVY2S6KUQA4C2MKIN3XKGFKRHZUDEITEGCRQITUATD6ZVANT2LW7
// CALCROAXSHD3HWE3O2EBJIGGWFMXD24725XIQL5P3IZHA6DE3ETO3NU2

const keypair = Keypair.fromSecret('SDNS3C4YQ5BDBXBZ56MEB754VLY352VSWGJZNW3W5CIUXEI3MEBTWKFT'); // GDT3KJMJIQWDOZWERC3K5SSGQGYALT2VSUAQP2YEGDK7YDPSQWUCIHYZ
const publicKey = keypair.publicKey()

// console.log([
//     ...keypair.rawSecretKey(),
//     ...keypair.rawPublicKey(),
// ])

// const nativeSAC = 'CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC'
const puzzleSAC = 'CALCROAXSHD3HWE3O2EBJIGGWFMXD24725XIQL5P3IZHA6DE3ETO3NU2'
const contractId = 'CAKX2ZMAKMID6PEDSUHBMU4NAHWUIEFXTXHESCSN7IRVG5E4QKAWSGLU'

const rpcUrl = 'https://soroban-testnet.stellar.org'
const rpc = new SorobanRpc.Server(rpcUrl)

const networkPassphrase = Networks.TESTNET

const contract = new Client({
    ...networks.testnet,
    ...basicNodeSigner(keypair, networkPassphrase),
    networkPassphrase,
    contractId,
    publicKey,
    rpcUrl,
})

const { built, simulationData } = await contract.call({
    sac: puzzleSAC
})

const entry = xdr.SorobanAuthorizationEntry.fromXDR(simulationData.result.auth[0].toXDR());
const credentials = entry.credentials().address();
const lastLedger = await rpc.getLatestLedger().then(({ sequence }) => sequence);
const preimage = xdr.HashIdPreimage.envelopeTypeSorobanAuthorization(
    new xdr.HashIdPreimageSorobanAuthorization({
        networkId: hash(Buffer.from(networkPassphrase)),
        nonce: credentials.nonce(),
        signatureExpirationLedger: lastLedger + DEFAULT_TIMEOUT,
        invocation: entry.rootInvocation()
    })
)
const payload = hash(preimage.toXDR())
const signature = keypair.sign(payload)

credentials.signatureExpirationLedger(lastLedger + DEFAULT_TIMEOUT)
credentials.signature(xdr.ScVal.scvMap([
    new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol('address'),
        val: Address.fromString(publicKey).toScVal()
    }),
    new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol('public_key'),
        val: xdr.ScVal.scvBytes(keypair.rawPublicKey()),
    }),
    new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol('signature'),
        val: xdr.ScVal.scvBytes(signature),
    }),
]))

const op = built?.operations[0] as Operation.InvokeHostFunction

op.auth?.splice(0, 1, entry)

console.log(built?.toXDR());

const sim = await rpc.simulateTransaction(built!)

if (
    SorobanRpc.Api.isSimulationError(sim)
    || SorobanRpc.Api.isSimulationRestore(sim)
) throw sim

const txn = SorobanRpc.assembleTransaction(new Transaction(built!.toXDR(), networkPassphrase), sim).build()

txn.sign(keypair)

const res = await rpc.sendTransaction(txn)

console.log(res);
