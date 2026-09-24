# evm-disassembler-rs

Turn contract bytecode (from `eth_getCode`) into readable opcodes.

```bash
cargo run -- 6080604052348015600f57600080fd5b50
cargo run -- --selectors $(cat code.hex)
```

- every opcode with its offset; PUSH1..PUSH32 print their immediate data
- `--selectors`: `PUSH4` values followed by `EQ` in the dispatcher, i.e. the functions the
  contract responds to
- Solidity appends CBOR metadata (`a2 64 'ipfs' ...` or `a1 65 'bzzr'...`) whose length is in
  the last two bytes; it's reported and not disassembled as code

Unknown bytes are shown as `INVALID(0x..)`.

```bash
cargo test
```
