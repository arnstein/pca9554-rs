//! PCA9554 Low-Voltage 8-Bit I2C and SMBus Low-Power I/O Expander
//!
//! https://www.ti.com/lit/ds/symlink/pca9554.pdf

// Tests require std for mocking the i2c bus
#![cfg_attr(not(test), no_std)]

use core::convert::TryFrom;
use core::marker::PhantomData;
use embedded_hal::blocking::i2c::{Write, WriteRead};

bitflags::bitflags! {
    pub struct Port: u8 {
        const P00 = 0b0000_0001;
        const P01 = 0b0000_0010;
        const P02 = 0b0000_0100;
        const P03 = 0b0000_1000;
        const P04 = 0b0001_0000;
        const P05 = 0b0010_0000;
        const P06 = 0b0100_0000;
        const P07 = 0b1000_0000;
    }
}
pub struct PCA9554<T> {
    address: Address,
    i2c: PhantomData<T>,
}

impl<T, E> PCA9554<T>
where
    T: WriteRead<Error = E> + Write<Error = E>,
{
    pub fn new(_i2c: &T, address: Address) -> Self {
        Self {
            address,
            i2c: PhantomData,
        }
    }

    pub fn address(&self) -> Address {
        self.address
    }

    /// Read a register.
    fn read(&self, i2c: &mut T, reg: Register) -> Result<Port, E> {
        let mut buffer = [0u8; 1];
        i2c.write_read(self.address as u8, &[reg as u8], &mut buffer)
            .map(|_| unsafe { Port::from_bits_unchecked(u8::from_le_bytes(buffer)) })
    }

    /// Write a register.
    fn write(&self, i2c: &mut T, reg: Register, port: Port) -> Result<(), E> {
        let bytes = port.bits.to_le_bytes();
        let buffer = [reg as u8, bytes[0]];
        i2c.write(self.address as u8, &buffer)
    }

    /// The Input Port register reflect the incoming logic levels of the pins, regardless of
    /// whether the pin is defined as an input or an output by the Configuration Register.
    pub fn read_inputs(&self, i2c: &mut T) -> Result<Port, E> {
        self.read(i2c, Register::INPUT_PORT)
    }

    /// The Output Port register show the outgoing logic levels of the pins defined as outputs
    /// by the Configuration Register.  These values reflect the state of the flip-flop controlling
    /// the output section, not the actual pin value.
    pub fn read_outputs(&self, i2c: &mut T) -> Result<Port, E> {
        self.read(i2c, Register::OUTPUT_PORT)
    }

    /// Set the output state for all pins configured as output pins in the Configuration Register.
    /// Has no effect for pins configured as input pins.
    ///
    /// To clear outputs use Port::empty() or the clear_outputs() method
    pub fn write_outputs(&self, i2c: &mut T, output: Port) -> Result<(), E> {
        self.write(i2c, Register::OUTPUT_PORT, output)
    }

    /// Set all outputs low.
    ///
    /// Equivalent to calling `PCA9554::write_outputs(i2c, Port::empty())`.
    pub fn clear_outputs(&self, i2c: &mut T) -> Result<(), E> {
        self.write(i2c, Register::OUTPUT_PORT, Port::empty())
    }

    /// Configure the direction of the I/O pins.  Ports set to 1 are configured as input pins with
    /// high-impedance output drivers.  Ports set to 0 are set as output pins.
    pub fn write_config(&self, i2c: &mut T, config: Port) -> Result<(), E> {
        self.write(i2c, Register::CONFIG_PORT, config)
    }

    /// Read the direction of the I/O pins.  Ports set to 1 are configured as input pins with
    /// high-impedance output drivers.  Ports set to 0 are set as output pins.
    pub fn read_config(&self, i2c: &mut T) -> Result<Port, E> {
        self.read(i2c, Register::CONFIG_PORT)
    }

    /// The Polarity Inversion register allow polarity inversion of pins defined as inputs by the
    /// Configuration register. If a bit in this register is set the corresponding pin's polarity
    /// is inverted. If a bit in this register is cleared, the corresponding pin's original polarity
    /// is retained.
    pub fn set_inverted(&self, i2c: &mut T, invert: Port) -> Result<(), E> {
        self.write(i2c, Register::POLARITY_INVERSION, invert)
    }

    /// The Polarity Inversion register allow polarity inversion of pins defined as inputs by the
    /// Configuration register. If a bit in this register is set the corresponding pin's polarity
    /// is inverted. If a bit in this register is cleared, the corresponding pin's original polarity
    /// is retained.
    pub fn is_inverted(&self, i2c: &mut T) -> Result<Port, E> {
        self.read(i2c, Register::POLARITY_INVERSION)
    }
}

/// Valid addresses for the PCA9554
#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Address {
    ADDR_0x20 = 0x20,
    ADDR_0x21 = 0x21,
    ADDR_0x22 = 0x22,
    ADDR_0x23 = 0x23,
    ADDR_0x24 = 0x24,
    ADDR_0x25 = 0x25,
    ADDR_0x26 = 0x26,
    ADDR_0x27 = 0x27,
}

impl TryFrom<u8> for Address {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x20 => Ok(Address::ADDR_0x20),
            0x21 => Ok(Address::ADDR_0x21),
            0x22 => Ok(Address::ADDR_0x22),
            0x23 => Ok(Address::ADDR_0x23),
            0x24 => Ok(Address::ADDR_0x24),
            0x25 => Ok(Address::ADDR_0x25),
            0x26 => Ok(Address::ADDR_0x26),
            0x27 => Ok(Address::ADDR_0x27),
            _ => Err(()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum Register {
    INPUT_PORT = 0x00,
    OUTPUT_PORT = 0x01,
    POLARITY_INVERSION = 0x02,
    CONFIG_PORT = 0x03,
    OUTPUT_DRIVE_0 = 0x40,
    OUTPUT_DRIVE_1 = 0x41,
    INPUT_LATCH = 0x42,
    PULLUPDOWN_EN = 0x43,
    PULLUPDOWN_SEL = 0x44,
    INTERRUPT_MASK = 0x45,
    INTERRUPT_STATUS = 0x46,
    OUTPUT_PORT_CONFIG = 0x4F,
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::i2c::{Mock, Transaction};

    #[test]
    fn test_read_inputs() {
        let addr = Address::ADDR_0x24;
        let expected_value = Port::P00;
        let expected = [Transaction::write_read(
            addr as u8,
            vec![Register::INPUT_PORT as u8],
            expected_value.bits.to_le_bytes().to_vec(),
        )];

        let mut i2c = Mock::new(&expected);
        let device = PCA9554::new(&i2c, addr);
        let result = device.read_inputs(&mut i2c).unwrap();
        assert_eq!(result, expected_value);
    }

    #[test]
    fn test_read_empty() {
        let addr = Address::ADDR_0x24;
        let raw_response_value = vec![0, 0];
        let expected = [Transaction::write_read(
            addr as u8,
            vec![Register::INPUT_PORT as u8],
            raw_response_value,
        )];
        let expected_result = Port::empty();

        let mut i2c = Mock::new(&expected);
        let device = PCA9554::new(&i2c, addr);
        let result = device.read_inputs(&mut i2c).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_read_outputs() {
        let addr = Address::ADDR_0x22;
        let raw_response_value = vec![0xAA, 0x55];
        let expected = [Transaction::write_read(
            addr as u8,
            vec![Register::OUTPUT_PORT as u8],
            raw_response_value,
        )];
        let expected_result = Port::P01
            | Port::P03
            | Port::P05
            | Port::P07;

        let mut i2c = Mock::new(&expected);
        let device = PCA9554::new(&i2c, addr);
        let result = device.read_outputs(&mut i2c).unwrap();
        assert_eq!(result, expected_result);
    }
}
