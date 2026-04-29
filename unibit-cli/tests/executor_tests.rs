use unibit_cli::executor::UnifiedExecutor;
use unibit_mustar::MotionPacket;

#[test]
fn test_unified_execution_logic() {
    // Lean execution for 10ms cap
    let mut exec = UnifiedExecutor::new();
    let mut pkt = MotionPacket::default();
    pkt.instruction_id = 0x1234;
    pkt.scope_count = 1;
    
    let denials = exec.execute_packet(&pkt);
    
    assert!(denials[0].is_admitted());
    assert_ne!(exec.receipt.0, 0);
}
