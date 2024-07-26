import { SorobanRpc, Networks, Keypair, xdr, hash, Address, Operation, Transaction, nativeToScVal } from '@stellar/stellar-sdk'
import { basicNodeSigner, DEFAULT_TIMEOUT } from '@stellar/stellar-sdk/contract';
import { Client as PuzzleClient, networks as puzzle_networks } from 'puzzle-sdk'
import { Client as SolverClient, networks as solver_networks } from 'solver-sdk'

// Issuer
// PUZZLE:GCUY5ECTBWZ5JGLXPPKSK4Y34RIMN2AEO4UNGQRRBF6AVVDSXSWLJLEN
// const issuer = Keypair.fromSecret('SB37H2EPZ4IK3JVLZPMMO3MYTFQ4UUXFZTS7VEHUOQJ4WVHCVMFOYRHB'); // GCUY5ECTBWZ5JGLXPPKSK4Y34RIMN2AEO4UNGQRRBF6AVVDSXSWLJLEN

// console.log([
//     ...issuer.rawSecretKey(),
//     ...issuer.rawPublicKey(),
// ])

const keypair = Keypair.fromSecret('SBMM4Q4A6AW2RFF7N62G62CT2ZYU4GZXK3EPPJFDJTWBRUDTFDT77VED'); // GCQXHDLSMF6YR53VNE6JXEBT3C53THISP2U2YDYESQG5BEBVBRNU4HZH
const publicKey = keypair.publicKey()

// console.log([
//     ...keypair.rawSecretKey(),
//     ...keypair.rawPublicKey(),
// ])

const nativeSAC = 'CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC'
const puzzleSAC = 'CDGOXJBEKI3MQDB3J477NN3HAQBDCNK5YYB2ZKAG24US53RXW44QIF6Z'
const puzzleId = 'CD6VH55NHALNPUS42TRXILJF26TZ4DAC42TKFJQVRXN7XG27X5EJCRKK'
const solverId = 'CBPHWYDHDNEM3B6CR3ZNBD4TRERGH2TTORW4GOVRGKVYWMR3GKSQOZIS'

const rpcUrl = 'https://soroban-testnet.stellar.org'
const rpc = new SorobanRpc.Server(rpcUrl)

const networkPassphrase = Networks.TESTNET

const puzzle = new PuzzleClient({
    ...puzzle_networks.testnet,
    ...basicNodeSigner(keypair, networkPassphrase),
    networkPassphrase,
    contractId: puzzleId,
    publicKey,
    rpcUrl,
})

const solver = new SolverClient({
    ...solver_networks.testnet,
    ...basicNodeSigner(keypair, networkPassphrase),
    networkPassphrase,
    contractId: solverId,
    publicKey,
    rpcUrl,
})

const { signAndSend } = await puzzle.setup({
    sac_in_address: nativeSAC,
    sac_out_address: puzzleSAC,
})

const setup = await signAndSend()

console.log(setup.getTransactionResponse?.status);

const { built, simulationData } = await solver.call({
    puzzle_address: puzzleId,
})

const invocation = new xdr.InvokeContractArgs({
    contractAddress: Address.fromString(nativeSAC).toScAddress(),
    functionName: "transfer",
    args: [
        nativeToScVal(publicKey, { type: 'address' }),
        nativeToScVal(puzzleId, { type: 'address' }),
        nativeToScVal(10_000_000, { type: 'i128' })
    ],
});

const transfer_entry = new xdr.SorobanAuthorizationEntry({
    credentials: xdr.SorobanCredentials.sorobanCredentialsSourceAccount(),
    rootInvocation: new xdr.SorobanAuthorizedInvocation({
        function: xdr.SorobanAuthorizedFunction.sorobanAuthorizedFunctionTypeContractFn(invocation),
        subInvocations: [],
    }),
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
op.auth?.push(transfer_entry)

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
