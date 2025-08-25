use std::io::{self, Write};

use crate::memory::PhyMemStatus;

#[derive(Debug, Default, Clone, Copy)]
pub struct FlatRegister {
    /// Clear to send(Modem related, not used)
    ///
    /// Bit 0
    pub cts: bool,

    /// Data set ready(Modem related, not used)
    ///
    /// Bit 1
    pub dsr: bool,

    /// Data carrier detected(Modem related, not used)
    ///
    /// Bit 2
    pub dcd: bool,

    /// UART busy. If this bit is set to 1, the UART is busy transmitting data.
    /// This bit remains set until the  complete byte, including all the stop
    /// bits, has been sent from the shift register.
    /// This bit is set as soon as the transmit FIFO becomes non-empty,
    /// regardless of whether the UART is  enabled or not.
    ///
    /// Bit 3
    pub busy: bool,

    /// Receive FIFO empty. The meaning of this bit depends on the state of the
    /// FEN bit in the UARTLCR_H  Register. If the FIFO is disabled, this bit is
    /// set when the receive holding register is empty. If the FIFO is enabled,
    /// the RXFE bit is set when the receive FIFO is empty.
    ///
    /// Bit 4
    pub rxfe: bool,

    /// Transmit FIFO full. The meaning of this bit depends on the state of the
    /// FEN bit in the UARTLCR_H  Register. If the FIFO is disabled, this bit is
    /// set when the transmit holding register is full. If the FIFO is enabled,
    /// the TXFF bit is set when the transmit FIFO is full.
    ///
    /// Bit 5
    pub txff: bool,

    /// Receive FIFO full. The meaning of this bit depends on the state of the
    /// FEN bit in the UARTLCR_H  Register. If the FIFO is disabled, this bit is
    /// set when the receive holding register is full. If the FIFO is enabled,
    /// the RXFF bit is set when the receive FIFO is full.
    ///
    /// Bit 6
    pub rxff: bool,

    /// Transmit FIFO empty. The meaning of this bit depends on the state of the
    /// FEN bit in the Line Control  Register, UARTLCR_H on page 3-12. If the
    /// FIFO is disabled, this bit is set when the transmit holding register is
    /// empty. If the FIFO is enabled, the TXFE bit is set when the transmit
    /// FIFO is empty. This bit does not indicate if there is data in the
    /// transmit shift register.
    ///
    /// Bit 7
    pub txfe: bool,

    /// Ring indicator (Modem related, not used)
    ///
    /// Bit 8
    pub ri: bool,
}
impl FlatRegister {
    pub fn to_bits(self) -> u16 {
        let mut v: u16 = 0;
        v |= self.cts as u16;
        v |= (self.dsr as u16) << 1;
        v |= (self.dcd as u16) << 2;
        v |= (self.busy as u16) << 3;
        v |= (self.rxfe as u16) << 4;
        v |= (self.txff as u16) << 5;
        v |= (self.rxff as u16) << 6;
        v |= (self.txfe as u16) << 7;
        v |= (self.ri as u16) << 8;
        v
    }

    #[inline]
    pub fn from_bits(bits: u16) -> Self {
        Self {
            cts: bits & 1 != 0,
            dsr: (bits >> 1) & 1 != 0,
            dcd: (bits >> 2) & 1 != 0,
            busy: (bits >> 3) & 1 != 0,
            rxfe: (bits >> 4) & 1 != 0,
            txff: (bits >> 5) & 1 != 0,
            rxff: (bits >> 6) & 1 != 0,
            txfe: (bits >> 7) & 1 != 0,
            ri: (bits >> 8) & 1 != 0,
        }
    }
}

/// PL011
///
/// https://developer.arm.com/documentation/ddi0183/latest/
#[derive(Debug, Default, Clone)]
pub struct Uart {
    /// Data Register, offset: 0x00
    pub dr: u8,
    /// Flat Register, offset: 0x18
    pub fr: FlatRegister,
}

impl Uart {
    pub fn read(&self, address: u8) -> (PhyMemStatus, u64) {
        match address {
            // DR
            0..=0x08 => self.read_dr(),
            0x18..=0x24 => self.read_fr(),
            _ => panic!("TODO!"),
        }
    }

    pub fn read_fr(&self) -> (PhyMemStatus, u64) {
        (PhyMemStatus::default(), self.fr.to_bits() as u64)
    }
    pub fn read_dr(&self) -> (PhyMemStatus, u64) {
        (PhyMemStatus::default(), self.dr as u64)
    }
    pub fn print(&mut self) {
        let mut stdio = io::stdout();
        stdio.write_all(&[self.dr]).expect("Failed to write to stdout.");
        stdio.flush().expect("Failed to flush stdout.");
    }
}
