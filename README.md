# evm-disassembler-rs

Turns contract bytecode (for example from `eth_getCode`) into readable opcodes.

```bash
cargo run -- 6080604052348015600f57600080fd5b50
cargo run -- --selectors $(cat code.hex)
```

Each opcode is printed with its offset, and PUSH1 to PUSH32 show their data. `--selectors` lists the
`PUSH4` values that are followed by `EQ` in the dispatcher, which are the function selectors the
contract responds to.

Solidity appends CBOR metadata (`a2 64 'ipfs' ...` or `a1 65 'bzzr' ...`) with its length in the last
two bytes. That part is reported separately instead of being disassembled. Unknown bytes show up as
`INVALID(0x..)`.

```bash
cargo test
```
