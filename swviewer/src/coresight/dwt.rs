//! Interface with the DWT (data watchpoint and trace) unit.
//!
//! This unit can monitor specific memory locations for write / read
//! access, this could be handy to debug a system :).
//!
//! See ARMv7-M architecture reference manual C1.8 for some additional
//! info about this stuff.

use super::Component;
use super::MemoryAccess;
use super::{CoreSightError, CoreSightResult};

pub const DWT_PID: [u8; 8] = [0x2, 0xB0, 0x3b, 0x0, 0x4, 0x0, 0x0, 0x0];

/// A struct representing a DWT unit on target.
pub struct Dwt<'m, M>
where
    M: MemoryAccess,
{
    component: Component<'m, M>,

    /// The number of comparators present in this device.
    num_comparators: usize,
}

const REG_OFFSET_DWT_CTRL: usize = 0;

impl<'m, M> Dwt<'m, M>
where
    M: MemoryAccess,
{
    pub fn new(component: Component<'m, M>) -> Self {
        Dwt {
            component,
            num_comparators: 0,
        }
    }

    pub fn info(&mut self) -> CoreSightResult<()> {
        let ctrl = self.component.read_reg(REG_OFFSET_DWT_CTRL)?;

        let num_comparators_available: u8 = ((ctrl >> 28) & 0xf) as u8;
        let has_trace_sampling_support = ctrl & (1 << 27) == 0;
        let has_compare_match_support = ctrl & (1 << 26) == 0;
        let has_cyccnt_support = ctrl & (1 << 25) == 0;
        let has_perf_counter_support = ctrl & (1 << 24) == 0;

        info!("DWT info:");
        info!(
            " number of comparators available: {}",
            num_comparators_available
        );
        self.num_comparators = num_comparators_available as usize;
        info!(" trace sampling support: {}", has_trace_sampling_support);
        info!(" compare match support: {}", has_compare_match_support);
        info!(" cyccnt support: {}", has_cyccnt_support);
        info!(" performance counter support: {}", has_perf_counter_support);
        Ok(())
    }

    pub fn setup_tracing(&self) -> CoreSightResult<()> {
        let mut value = self.component.read_reg(REG_OFFSET_DWT_CTRL)?;
        value |= 1 << 10; // Sync packet rate.
        value |= 1 << 0; // Enable CYCCNT.
        self.component.write_reg(REG_OFFSET_DWT_CTRL, value)?;
        Ok(())
    }

    /// Enable data monitor on a given user variable at some address
    pub fn enable_trace(&self, var_address: u32, comparator: usize) -> CoreSightResult<()> {
        if comparator < self.num_comparators {
            let mask = 0; // size of the ignore mask, ignore nothing!
                          // 3 - sample PC and data (and Read/Write access)
                          // 0b1101 = 13 - sample data (write only access)
                          //
                          // function |= 0b10 << 10; // COMP register contains word sized unit.
            let function: u32 = 13;

            // entry x:
            self.component
                .write_reg(0x20 + comparator * 16, var_address)?; // COMp value
            self.component.write_reg(0x24 + comparator * 16, mask)?; // mask
            self.component.write_reg(0x28 + comparator * 16, function)?; // function

            Ok(())
        } else {
            Err(CoreSightError::Other(format!(
                "Channel out of bounds: {}",
                comparator
            )))
        }
    }

    pub fn disable_memory_watch(&self, channel: usize) -> CoreSightResult<()> {
        if channel < self.num_comparators {
            self.component.write_reg(0x28 + channel * 16, 0)?; // function, 0 is disabled.
            Ok(())
        } else {
            Err(CoreSightError::Other(format!(
                "Channel out of bounds: {}",
                channel
            )))
        }
    }

    pub fn poll(&self) -> CoreSightResult<()> {
        let status = self.component.read_reg(0x28)?;
        let matched = status & (1 << 24) > 0;
        info!("DWT function0 State: matched={}", matched);
        Ok(())
    }
}
