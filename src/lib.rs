pub fn name(op: u8) -> String {
    let fixed = match op {
        0x00 => "STOP", 0x01 => "ADD", 0x02 => "MUL", 0x03 => "SUB", 0x04 => "DIV", 0x05 => "SDIV", 0x06 => "MOD",
        0x07 => "SMOD", 0x08 => "ADDMOD", 0x09 => "MULMOD", 0x0a => "EXP", 0x0b => "SIGNEXTEND",
        0x10 => "LT", 0x11 => "GT", 0x12 => "SLT", 0x13 => "SGT", 0x14 => "EQ", 0x15 => "ISZERO", 0x16 => "AND",
        0x17 => "OR", 0x18 => "XOR", 0x19 => "NOT", 0x1a => "BYTE", 0x1b => "SHL", 0x1c => "SHR", 0x1d => "SAR",
        0x20 => "KECCAK256", 0x30 => "ADDRESS", 0x31 => "BALANCE", 0x32 => "ORIGIN", 0x33 => "CALLER",
        0x34 => "CALLVALUE", 0x35 => "CALLDATALOAD", 0x36 => "CALLDATASIZE", 0x37 => "CALLDATACOPY",
        0x38 => "CODESIZE", 0x39 => "CODECOPY", 0x3a => "GASPRICE", 0x3b => "EXTCODESIZE", 0x3c => "EXTCODECOPY",
        0x3d => "RETURNDATASIZE", 0x3e => "RETURNDATACOPY", 0x3f => "EXTCODEHASH", 0x40 => "BLOCKHASH",
        0x41 => "COINBASE", 0x42 => "TIMESTAMP", 0x43 => "NUMBER", 0x44 => "PREVRANDAO", 0x45 => "GASLIMIT",
        0x46 => "CHAINID", 0x47 => "SELFBALANCE", 0x48 => "BASEFEE", 0x49 => "BLOBHASH", 0x4a => "BLOBBASEFEE",
        0x50 => "POP", 0x51 => "MLOAD", 0x52 => "MSTORE", 0x53 => "MSTORE8", 0x54 => "SLOAD", 0x55 => "SSTORE",
        0x56 => "JUMP", 0x57 => "JUMPI", 0x58 => "PC", 0x59 => "MSIZE", 0x5a => "GAS", 0x5b => "JUMPDEST",
        0x5c => "TLOAD", 0x5d => "TSTORE", 0x5e => "MCOPY", 0x5f => "PUSH0",
        0xf0 => "CREATE", 0xf1 => "CALL", 0xf2 => "CALLCODE", 0xf3 => "RETURN", 0xf4 => "DELEGATECALL",
        0xf5 => "CREATE2", 0xfa => "STATICCALL", 0xfd => "REVERT", 0xfe => "INVALID", 0xff => "SELFDESTRUCT",
        _ => "",
    };
    if !fixed.is_empty() {
        return fixed.to_string();
    }
    match op {
        0x60..=0x7f => format!("PUSH{}", op - 0x5f),
        0x80..=0x8f => format!("DUP{}", op - 0x7f),
        0x90..=0x9f => format!("SWAP{}", op - 0x8f),
        0xa0..=0xa4 => format!("LOG{}", op - 0xa0),
        _ => format!("INVALID(0x{op:02x})"),
    }
}

pub struct Instr {
    pub offset: usize,
    pub op: u8,
    pub data: Vec<u8>,
}

/// Length of Solidity's CBOR metadata tail (including the 2 length bytes), if present.
pub fn metadata_len(code: &[u8]) -> Option<usize> {
    if code.len() < 2 {
        return None;
    }
    let n = u16::from_be_bytes([code[code.len() - 2], code[code.len() - 1]]) as usize + 2;
    let start = code.len().checked_sub(n)?;
    matches!(code.get(start), Some(0xa1) | Some(0xa2)).then_some(n)
}

pub fn disassemble(code: &[u8]) -> Vec<Instr> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < code.len() {
        let op = code[i];
        let n = if (0x60..=0x7f).contains(&op) { (op - 0x5f) as usize } else { 0 };
        let end = (i + 1 + n).min(code.len());
        out.push(Instr { offset: i, op, data: code[i + 1..end].to_vec() });
        i += 1 + n;
    }
    out
}

/// PUSH4 immediates directly followed by EQ: the function selector dispatch table.
pub fn selectors(ins: &[Instr]) -> Vec<String> {
    let mut out = Vec::new();
    for w in ins.windows(2) {
        if w[0].op == 0x63 && w[1].op == 0x14 {
            let s: String = w[0].data.iter().map(|b| format!("{b:02x}")).collect();
            if !out.contains(&s) {
                out.push(s);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disassembles_push_data() {
        let ins = disassemble(&[0x60, 0x80, 0x60, 0x40, 0x52]);
        assert_eq!(ins.len(), 3);
        assert_eq!(name(ins[0].op), "PUSH1");
        assert_eq!(ins[1].data, vec![0x40]);
        assert_eq!(name(ins[2].op), "MSTORE");
    }

    #[test]
    fn finds_selectors() {
        let code = [0x63, 0xa9, 0x05, 0x9c, 0xbb, 0x14, 0x61, 0x00, 0x10, 0x57];
        assert_eq!(selectors(&disassemble(&code)), vec!["a9059cbb".to_string()]);
    }

    #[test]
    fn names_ranges() {
        assert_eq!(name(0x7f), "PUSH32");
        assert_eq!(name(0x8f), "DUP16");
        assert_eq!(name(0xa2), "LOG2");
        assert_eq!(name(0x0c), "INVALID(0x0c)");
    }

    #[test]
    fn metadata_tail() {
        let mut code = vec![0x00, 0xa2, 0x01, 0x02];
        code.extend_from_slice(&[0x00, 0x03]); // tail length 3 (a2 01 02)
        assert_eq!(metadata_len(&code), Some(5));
        assert_eq!(metadata_len(&[0x00, 0x01]), None);
    }
}
