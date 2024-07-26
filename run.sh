rm -rf target/wasm32-unknown-unknown/release

stellar contract build

puzzle=$(stellar contract deploy --wasm target/wasm32-unknown-unknown/release/puzzle.wasm --network testnet --source default)
solver=$(stellar contract deploy --wasm target/wasm32-unknown-unknown/release/solver.wasm --network testnet --source default)

echo "Puzzle contract: $puzzle"
echo "Solver contract: $solver"

stellar contract invoke --id CDGOXJBEKI3MQDB3J477NN3HAQBDCNK5YYB2ZKAG24US53RXW44QIF6Z --network testnet --source SB37H2EPZ4IK3JVLZPMMO3MYTFQ4UUXFZTS7VEHUOQJ4WVHCVMFOYRHB -- mint --to $puzzle --amount 170141183460469231731687303715884105727
stellar contract invoke --id $puzzle --network testnet --source default -- setup --sac_in_address CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC --sac_out_address CDGOXJBEKI3MQDB3J477NN3HAQBDCNK5YYB2ZKAG24US53RXW44QIF6Z

stellar contract bindings typescript --network testnet --contract-id $puzzle --output-dir ./bun_tests/puzzle-sdk --overwrite
stellar contract bindings typescript --network testnet --contract-id $solver --output-dir ./bun_tests/solver-sdk --overwrite

cd ./bun_tests && bun install --force