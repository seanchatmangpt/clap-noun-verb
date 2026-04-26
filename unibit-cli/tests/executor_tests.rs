use unibit_cli::executor::UnifiedExecutor;
use unibit_mustar::MotionPacket;

#[test]
fn test_unified_execution_logic() {
    let mut exec = UnifiedExecutor::new();
    
    // A motion packet with a non-zero instruction_id should cause receipts to advance
    let mut pkt = MotionPacket::default();
    pkt.instruction_id = 0x1234;
    pkt.scope_count = 1; // Address at least one word
    
    let denials = exec.execute_packet(&pkt);
    
    // In unibit kernel, zero means admitted
    for (i, denial) in denials.iter().enumerate() {
        assert!(denial.is_admitted(), "Instruction {} denied: {:?}", i, denial);
    }
    
    // Receipts should have advanced from genesis
    assert_ne!(exec.receipt.0, 0, "FNV-1a receipt did not advance");
    assert_ne!(exec.causal_receipt, unibit_causality::UCausalReceipt::genesis(), "BLAKE3 causal receipt did not advance");
}
