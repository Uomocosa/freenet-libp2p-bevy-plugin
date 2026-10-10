#[must_use]
pub const fn contract_wasm() -> &'static [u8] {
    include_bytes!("../../../contract/lobby/lobby_contract.wasm")
}

#[cfg(test)]
mod tests {
    use super::contract_wasm;

    #[test]
    fn test_usage() {
        let wasm = contract_wasm();
        assert!(wasm.len() > 4);
        assert_eq!(&wasm[0..4], b"\0asm");
    }
}
