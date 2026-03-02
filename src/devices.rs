use std::io::{self, Write};

use crate::utils::RingBuf;

const FIFO_CAPACITY: usize = 16;

use crate::memory::PhyMemStatus;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
    /// FEN bit in the UARTLCR_H  Register. If the FIFO is disabled, this bit
    /// is set when the receive holding register is empty. If the FIFO is
    /// enabled, the RXFE bit is set when the receive FIFO is empty.
    ///
    /// Bit 4
    pub rxfe: bool = true,

    /// Transmit FIFO full. The meaning of this bit depends on the state of the
    /// FEN bit in the UARTLCR_H  Register. If the FIFO is disabled, this bit
    /// is set when the transmit holding register is full. If the FIFO is
    /// enabled, the TXFF bit is set when the transmit FIFO is full.
    ///
    /// Bit 5
    pub txff: bool,

    /// Receive FIFO full. The meaning of this bit depends on the state of the
    /// FEN bit in the UARTLCR_H  Register. If the FIFO is disabled, this bit
    /// is set when the receive holding register is full. If the FIFO is
    /// enabled, the RXFF bit is set when the receive FIFO is full.
    ///
    /// Bit 6
    pub rxff: bool,

    /// Transmit FIFO empty. The meaning of this bit depends on the state of
    /// the FEN bit in the Line Control  Register, UARTLCR_H on page 3-12.
    /// If the FIFO is disabled, this bit is set when the transmit holding
    /// register is empty. If the FIFO is enabled, the TXFE bit is set when
    /// the transmit FIFO is empty. This bit does not indicate if there is
    /// data in the transmit shift register.
    ///
    /// Bit 7
    pub txfe: bool = true,

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

/// Interrupt Mask Set/Clear Register, UARTIMSC
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Imsc {
    /// nUARTRI modem interrupt mask
    ///
    /// Bit 0
    pub rimim: bool,

    /// nUARTCTS modem interrupt mask
    ///
    /// Bit 1
    pub ctsmim: bool,

    /// nUARTDCD modem interrupt mask
    ///
    /// Bit 2
    pub dcdmim: bool,

    /// nUARTDSR modem interrupt mask
    ///
    /// Bit 3
    pub dsrmim: bool,

    /// Receive interrupt mask
    ///
    /// Bit 4
    pub rxim: bool,

    /// Transmit interrupt mask
    ///
    /// Bit 5
    pub txim: bool,

    /// Receive timeout interrupt mask
    ///
    /// Bit 6
    pub rtim: bool,

    /// Framing error interrupt mask
    ///
    /// Bit 7
    pub feim: bool,

    /// Parity error interrupt mask
    ///
    /// Bit 8
    pub peim: bool,

    /// Break error interrupt mask
    ///
    /// Bit 9
    pub beim: bool,

    /// Overrun error interrupt mask
    ///
    /// Bit 10
    pub oeim: bool,
}

impl Imsc {
    pub fn to_bits(self) -> u16 {
        let mut v: u16 = 0;
        v |= self.rimim as u16;
        v |= (self.ctsmim as u16) << 1;
        v |= (self.dcdmim as u16) << 2;
        v |= (self.dsrmim as u16) << 3;
        v |= (self.rxim as u16) << 4;
        v |= (self.txim as u16) << 5;
        v |= (self.rtim as u16) << 6;
        v |= (self.feim as u16) << 7;
        v |= (self.peim as u16) << 8;
        v |= (self.beim as u16) << 9;
        v |= (self.oeim as u16) << 10;
        v
    }

    #[inline]
    pub fn from_bits(bits: u16) -> Self {
        Self {
            rimim: bits & 1 != 0,
            ctsmim: (bits >> 1) & 1 != 0,
            dcdmim: (bits >> 2) & 1 != 0,
            dsrmim: (bits >> 3) & 1 != 0,
            rxim: (bits >> 4) & 1 != 0,
            txim: (bits >> 5) & 1 != 0,
            rtim: (bits >> 6) & 1 != 0,
            feim: (bits >> 7) & 1 != 0,
            peim: (bits >> 8) & 1 != 0,
            beim: (bits >> 9) & 1 != 0,
            oeim: (bits >> 10) & 1 != 0,
        }
    }
}

/// Interrupt Clear Register, UARTICR (write-only)
///
/// Writing 1 to a bit clears the corresponding interrupt in RIS.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Icr {
    /// nUARTRI modem interrupt clear
    ///
    /// Bit 0
    pub rimic: bool,

    /// nUARTCTS modem interrupt clear
    ///
    /// Bit 1
    pub ctsmic: bool,

    /// nUARTDCD modem interrupt clear
    ///
    /// Bit 2
    pub dcdmic: bool,

    /// nUARTDSR modem interrupt clear
    ///
    /// Bit 3
    pub dsrmic: bool,

    /// Receive interrupt clear
    ///
    /// Bit 4
    pub rxic: bool,

    /// Transmit interrupt clear
    ///
    /// Bit 5
    pub txic: bool,

    /// Receive timeout interrupt clear
    ///
    /// Bit 6
    pub rtic: bool,

    /// Framing error interrupt clear
    ///
    /// Bit 7
    pub feic: bool,

    /// Parity error interrupt clear
    ///
    /// Bit 8
    pub peic: bool,

    /// Break error interrupt clear
    ///
    /// Bit 9
    pub beic: bool,

    /// Overrun error interrupt clear
    ///
    /// Bit 10
    pub oeic: bool,
}

impl Icr {
    #[inline]
    pub fn from_bits(bits: u16) -> Self {
        Self {
            rimic: bits & 1 != 0,
            ctsmic: (bits >> 1) & 1 != 0,
            dcdmic: (bits >> 2) & 1 != 0,
            dsrmic: (bits >> 3) & 1 != 0,
            rxic: (bits >> 4) & 1 != 0,
            txic: (bits >> 5) & 1 != 0,
            rtic: (bits >> 6) & 1 != 0,
            feic: (bits >> 7) & 1 != 0,
            peic: (bits >> 8) & 1 != 0,
            beic: (bits >> 9) & 1 != 0,
            oeic: (bits >> 10) & 1 != 0,
        }
    }

    pub fn to_bits(self) -> u16 {
        let mut v: u16 = 0;
        v |= self.rimic as u16;
        v |= (self.ctsmic as u16) << 1;
        v |= (self.dcdmic as u16) << 2;
        v |= (self.dsrmic as u16) << 3;
        v |= (self.rxic as u16) << 4;
        v |= (self.txic as u16) << 5;
        v |= (self.rtic as u16) << 6;
        v |= (self.feic as u16) << 7;
        v |= (self.peic as u16) << 8;
        v |= (self.beic as u16) << 9;
        v |= (self.oeic as u16) << 10;
        v
    }
}

/// Control Register, UARTCR
///
/// Reset value: 0x0300 (TXE and RXE set to 1)
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cr {
    /// UART enable
    ///
    /// Bit 0
    pub uarten: bool,

    /// SIR enable
    ///
    /// Bit 1
    pub siren: bool,

    /// SIR low-power IrDA mode
    ///
    /// Bit 2
    pub sirlp: bool,

    // Bits 3:6 reserved
    /// Loopback enable
    ///
    /// Bit 7
    pub lbe: bool,

    /// Transmit enable
    ///
    /// Bit 8
    pub txe: bool,

    /// Receive enable
    ///
    /// Bit 9
    pub rxe: bool,

    /// Data transmit ready
    ///
    /// Bit 10
    pub dtr: bool,

    /// Request to send
    ///
    /// Bit 11
    pub rts: bool,

    /// Out1 (DCD)
    ///
    /// Bit 12
    pub out1: bool,

    /// Out2 (RI)
    ///
    /// Bit 13
    pub out2: bool,

    /// RTS hardware flow control enable
    ///
    /// Bit 14
    pub rtsen: bool,

    /// CTS hardware flow control enable
    ///
    /// Bit 15
    pub ctsen: bool,
}

impl Default for Cr {
    fn default() -> Self {
        Self {
            uarten: false,
            siren: false,
            sirlp: false,
            lbe: false,
            txe: true,
            rxe: true,
            dtr: false,
            rts: false,
            out1: false,
            out2: false,
            rtsen: false,
            ctsen: false,
        }
    }
}

impl Cr {
    pub fn to_bits(self) -> u16 {
        let mut v: u16 = 0;
        v |= self.uarten as u16;
        v |= (self.siren as u16) << 1;
        v |= (self.sirlp as u16) << 2;
        // bits 3:6 reserved
        v |= (self.lbe as u16) << 7;
        v |= (self.txe as u16) << 8;
        v |= (self.rxe as u16) << 9;
        v |= (self.dtr as u16) << 10;
        v |= (self.rts as u16) << 11;
        v |= (self.out1 as u16) << 12;
        v |= (self.out2 as u16) << 13;
        v |= (self.rtsen as u16) << 14;
        v |= (self.ctsen as u16) << 15;
        v
    }

    #[inline]
    pub fn from_bits(bits: u16) -> Self {
        Self {
            uarten: bits & 1 != 0,
            siren: (bits >> 1) & 1 != 0,
            sirlp: (bits >> 2) & 1 != 0,
            lbe: (bits >> 7) & 1 != 0,
            txe: (bits >> 8) & 1 != 0,
            rxe: (bits >> 9) & 1 != 0,
            dtr: (bits >> 10) & 1 != 0,
            rts: (bits >> 11) & 1 != 0,
            out1: (bits >> 12) & 1 != 0,
            out2: (bits >> 13) & 1 != 0,
            rtsen: (bits >> 14) & 1 != 0,
            ctsen: (bits >> 15) & 1 != 0,
        }
    }
}

/// Line Control Register, UARTLCR_H, offset: 0x2C
///
/// Note: writing LCR_H latches the staged IBRD/FBRD values (irrelevant in
/// emulator).
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LcrH {
    /// Send break
    ///
    /// Bit 0
    pub brk: bool,

    /// Parity enable
    ///
    /// Bit 1
    pub pen: bool,

    /// Even parity select
    ///
    /// Bit 2
    pub eps: bool,

    /// Two stop bits select
    ///
    /// Bit 3
    pub stp2: bool,

    /// Enable FIFOs (0 = 1-byte holding registers, 1 = FIFO mode)
    ///
    /// Bit 4
    pub fen: bool,

    /// Word length: 0b00=5, 0b01=6, 0b10=7, 0b11=8
    ///
    /// Bits 6:5
    pub wlen: u8,

    /// Stick parity select
    ///
    /// Bit 7
    pub sps: bool,
}

impl LcrH {
    pub fn to_bits(self) -> u16 {
        let mut v: u16 = 0;
        v |= self.brk as u16;
        v |= (self.pen as u16) << 1;
        v |= (self.eps as u16) << 2;
        v |= (self.stp2 as u16) << 3;
        v |= (self.fen as u16) << 4;
        v |= (self.wlen as u16 & 0x3) << 5;
        v |= (self.sps as u16) << 7;
        v
    }

    #[inline]
    pub fn from_bits(bits: u16) -> Self {
        Self {
            brk: bits & 1 != 0,
            pen: (bits >> 1) & 1 != 0,
            eps: (bits >> 2) & 1 != 0,
            stp2: (bits >> 3) & 1 != 0,
            fen: (bits >> 4) & 1 != 0,
            wlen: ((bits >> 5) & 0x3) as u8,
            sps: (bits >> 7) & 1 != 0,
        }
    }
}

/// PL011
///
/// https://developer.arm.com/documentation/ddi0183/latest/
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Uart {
    /// Data Register, offset: 0x00
    pub dr: u8,
    pub rx_fifo: RingBuf<FIFO_CAPACITY>,
    /// Flat Register, offset: 0x18
    pub fr: FlatRegister,

    /// FIFO Interrupt threshold
    /// TXIFLSEL [2:0]
    /// RXIFLSEL [5:3]
    pub ifls: u8 = 0x12,
    /// Integer Baud Rate Register, offset: 0x24 (no functional effect in
    /// emulator)
    pub ibrd: u16,
    /// Fractional Baud Rate Register, offset: 0x28 (no functional effect in
    /// emulator)
    pub fbrd: u8,
    /// Line Control Register, offset: 0x2C
    pub lcr_h: LcrH,
    /// Control Register, offset: 0x30
    pub cr: Cr,
    /// Interrupt Mask Set/Clear Register, offset: 0x38
    pub imsc: Imsc,
    /// Raw Interrupt Status Register, offset: 0x3C
    pub ris: u16,
}

impl Uart {
    pub fn read(&mut self, address: u16) -> (PhyMemStatus, u64) {
        match address {
            // DR
            0..=0x08 => self.read_dr(),
            // FR
            0x18 => self.read_fr(),
            // IBRD
            0x24 => (PhyMemStatus::default(), self.ibrd as u64),
            // FBRD
            0x28 => (PhyMemStatus::default(), self.fbrd as u64),
            // LCR_H
            0x2C => (PhyMemStatus::default(), self.lcr_h.to_bits() as u64),
            // CR
            0x30 => self.read_cr(),
            // IFLS
            0x34 => (PhyMemStatus::default(), self.ifls as u64),
            // IMSC
            0x38 => self.read_imsc(),
            // RIS
            0x3C => (PhyMemStatus::default(), self.ris as u64),
            // MIS = RIS & IMSC
            0x40 => (PhyMemStatus::default(), (self.ris & self.imsc.to_bits()) as u64),
            0x090..=0xFCC => (PhyMemStatus::default(), 0),
            0xFE0 => (PhyMemStatus::default(), 0x11),
            0xFE4 => (PhyMemStatus::default(), 0x10),
            0xFE8 => (PhyMemStatus::default(), 0x14),
            0xFEC => (PhyMemStatus::default(), 0x0),
            0xFF0 => (PhyMemStatus::default(), 0x0D),
            0xFF4 => (PhyMemStatus::default(), 0xF0),
            0xFF8 => (PhyMemStatus::default(), 0x05),
            0xFFC => (PhyMemStatus::default(), 0xB1),
            _ => panic!("Tried to read UART address: {address:#03x}"),
        }
    }

    pub fn write(&mut self, address: u16, value: u64, _size: usize) -> PhyMemStatus {
        let value: u16 = value.try_into().expect("");
        match address {
            // DR
            0..=0x08 => {
                self.dr = value as u8;
                PhyMemStatus::default()
            }
            // IBRD
            0x24 => {
                self.ibrd = value;
                PhyMemStatus::default()
            }
            // FBRD
            0x28 => {
                self.fbrd = (value & 0x3F) as u8;
                PhyMemStatus::default()
            }
            // LCR_H
            0x2C => {
                self.lcr_h = LcrH::from_bits(value);
                PhyMemStatus::default()
            }
            // CR
            0x30 => {
                self.cr = Cr::from_bits(value);
                PhyMemStatus::default()
            }
            // IFLS
            0x34 => {
                self.ifls = value as u8;
                PhyMemStatus::default()
            }
            // IMSC
            0x38 => {
                self.imsc = Imsc::from_bits(value);
                PhyMemStatus::default()
            }
            // ICR (write-only, clears bits in RIS)
            0x44 => {
                self.ris &= !value;
                PhyMemStatus::default()
            }
            // FR
            0x18 => panic!("UART FlatRegister can't be written"),
            _ => panic!("Tried to write UART address: {address:#03x}"),
        }
    }

    pub fn read_fr(&self) -> (PhyMemStatus, u64) {
        (PhyMemStatus::default(), self.fr.to_bits() as u64)
    }
    pub fn read_dr(&mut self) -> (PhyMemStatus, u64) {
        let byte = self.rx_fifo.pop_front().unwrap_or(0);
        self.fr.rxfe = self.rx_fifo.is_empty();
        self.fr.rxff = false;
        (PhyMemStatus::default(), byte as u64)
    }

    pub fn read_cr(&self) -> (PhyMemStatus, u64) {
        (PhyMemStatus::default(), self.cr.to_bits() as u64)
    }

    pub fn read_imsc(&self) -> (PhyMemStatus, u64) {
        (PhyMemStatus::default(), self.imsc.to_bits() as u64)
    }

    pub fn print(&mut self) {
        let mut stdio = io::stdout();
        stdio.write_all(&[self.dr]).expect("Failed to write to stdout.");
        stdio.flush().expect("Failed to flush stdout.");
    }
}
