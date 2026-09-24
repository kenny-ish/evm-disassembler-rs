use evm_disassembler_rs::{disassemble, metadata_len, name, selectors};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let only_selectors = args.iter().any(|a| a == "--selectors");
    let Some(hex) = args.iter().find(|a| !a.starts_with("--")) else {
        eprintln!("usage: evm-disassembler-rs [--selectors] BYTECODE_HEX");
        std::process::exit(2);
    };
    let hex = hex.trim_start_matches("0x");
    let code: Vec<u8> = match (0..hex.len()).step_by(2).map(|i| u8::from_str_radix(hex.get(i..i + 2).unwrap_or("x"), 16)).collect() {
        Ok(c) => c,
        Err(_) => {
            eprintln!("invalid hex");
            std::process::exit(1);
        }
    };
    let meta = metadata_len(&code);
    let body = &code[..code.len() - meta.unwrap_or(0)];
    let ins = disassemble(body);
    if only_selectors {
        for s in selectors(&ins) {
            println!("0x{s}");
        }
        return;
    }
    for i in &ins {
        let data: String = i.data.iter().map(|b| format!("{b:02x}")).collect();
        println!("{:05x}  {:<12} {}", i.offset, name(i.op), if data.is_empty() { String::new() } else { format!("0x{data}") });
    }
    if let Some(n) = meta {
        println!("\n{n} bytes of Solidity metadata (CBOR) at the end");
    }
}
