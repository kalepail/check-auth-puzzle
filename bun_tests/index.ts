import { SorobanRpc, Networks, Keypair, xdr, hash, Address, Operation, Transaction } from '@stellar/stellar-sdk'
import { basicNodeSigner, DEFAULT_TIMEOUT } from '@stellar/stellar-sdk/contract';
import { Client, networks } from 'puzzle-sdk'

// Issuer
// PUZZLE
// CDGOXJBEKI3MQDB3J477NN3HAQBDCNK5YYB2ZKAG24US53RXW44QIF6Z
// GCUY5ECTBWZ5JGLXPPKSK4Y34RIMN2AEO4UNGQRRBF6AVVDSXSWLJLEN
// SB37H2EPZ4IK3JVLZPMMO3MYTFQ4UUXFZTS7VEHUOQJ4WVHCVMFOYRHB

const keypair = Keypair.fromSecret('SBMM4Q4A6AW2RFF7N62G62CT2ZYU4GZXK3EPPJFDJTWBRUDTFDT77VED'); // GCQXHDLSMF6YR53VNE6JXEBT3C53THISP2U2YDYESQG5BEBVBRNU4HZH
const publicKey = keypair.publicKey()

console.log([
    ...keypair.rawSecretKey(),
    ...keypair.rawPublicKey(),
])

const puzzleSAC = 'CDGOXJBEKI3MQDB3J477NN3HAQBDCNK5YYB2ZKAG24US53RXW44QIF6Z'
const puzzleId = 'CCPYY3EQZQ6SQE2XRHCU5VVH4DCR3ZCNRHIS5ITCSMBK2WOPMN56LEAV'
const solverId = 'CD3QSVJ2FLC4XMRHPVS3NM2TFFKXNTGINP3A55W5P5FKZR5YHS6PABT2'

const rpcUrl = 'https://soroban-testnet.stellar.org'
const rpc = new SorobanRpc.Server(rpcUrl)

const networkPassphrase = Networks.TESTNET

const contract = new Client({
    ...networks.testnet,
    ...basicNodeSigner(keypair, networkPassphrase),
    networkPassphrase,
    contractId: solverId,
    publicKey,
    rpcUrl,
})

const { built, simulationData } = await contract.call({
    puzzle: puzzleId,
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
        key: xdr.ScVal.scvSymbol('signature'),
        val: xdr.ScVal.scvBytes(signature),
    }),
]))

const op = built?.operations[0] as Operation.InvokeHostFunction

op.auth?.splice(0, 1, entry)

console.log('\n', built?.toXDR(), '\n');

const sim = await rpc.simulateTransaction(built!)

if (SorobanRpc.Api.isSimulationError(sim)) 
    throw sim.error

if (SorobanRpc.Api.isSimulationRestore(sim))
    throw 'Restore required'

const txn = SorobanRpc.assembleTransaction(new Transaction(built!.toXDR(), networkPassphrase), sim).build()

txn.sign(keypair)

console.log('\n', txn.toXDR(), '\n');

const res = await rpc.sendTransaction(txn)

console.log(res);
