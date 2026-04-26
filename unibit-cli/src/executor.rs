//! Unified POWL64 Executor implementation

use unibit_kernel::{UCell, UDenial, UReceipt};
use unibit_l1::L1Region;
use unibit_causality::UCausalReceipt;
use unibit_mustar::MotionPacket;

pub struct UnifiedExecutor {
    pub region: Box<L1Region>,
    pub receipt: UReceipt,
    pub causal_receipt: UCausalReceipt,
}

impl UnifiedExecutor {
    pub fn new() -> Self {
        Self {
            region: Box::new(L1Region::default()),
            receipt: UReceipt(0), // FNV-1a genesis
            causal_receipt: UCausalReceipt::genesis(),
        }
    }

    /// Execute a single motion packet
    pub fn execute_packet(&mut self, pkt: &MotionPacket) -> Vec<UDenial> {
        let mut denials = Vec::new();
        
        // A MotionPacket addresses multiple scope slots (TruthBlock words)
        for i in 0..pkt.scope_count as usize {
            let instr = pkt.to_instruction(i);
            let word_index = instr.scope as u32;
            
            // Extract current state from region via read_word
            let word_val = self.region.truth.read_word(word_index)
                .expect("TruthBlock read out of bounds");
            let current_state = UCell(word_val);
            
            // Execute kernel step
            let (new_state, denial, delta, next_receipt) = unibit_kernel::execute_step(
                current_state,
                word_index,
                &instr,
                self.receipt
            );
            
            // Update region truth via write_word
            self.region.truth.write_word(word_index, new_state.0)
                .expect("TruthBlock write out of bounds");
            
            // Update rolling receipts
            self.receipt = next_receipt;
            
            // Update causal receipt (BLAKE3)
            if denial.is_admitted() {
                self.causal_receipt = unibit_causality::causal_mix(
                    self.causal_receipt,
                    instr.id,
                    instr.scope.try_into().unwrap_or(0),
                    new_state.0 ^ current_state.0, // Fired mask (XOR delta)
                    &delta
                );
            }
            
            denials.push(denial);
        }
        
        denials
    }
}

impl Default for UnifiedExecutor {
    fn default() -> Self {
        Self::new()
    }
}
