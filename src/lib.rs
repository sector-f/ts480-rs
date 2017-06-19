extern crate ascii;
use ascii::{AsciiString, ToAsciiChar};

extern crate serial;
use serial::{SystemPort, SerialPort};

use std::io::{Read, Write};
use std::ffi::{OsStr, OsString};

pub type RadioResult<T> = serial::Result<Result<T, RadioError>>;

pub struct TS480 {
    port: SystemPort,
    port_name: OsString,
}

impl TS480 {
    /// Attempts to connect to the radio using the specified port.
    /// On *nix systems, this should be a device file, such as `/dev/ttyS0`.
    /// On Windows, this should be a COM port, such as `COM1`
    pub fn new<T: AsRef<OsStr> + ?Sized>(port: &T) -> serial::Result<Self> {
        let serial_port = serial::open(port)?;
        Ok(TS480 {
            port: serial_port,
            port_name: OsString::from(port),
        })
    }

    /// Attempts to reconnect to the radio using the originally-specified port
    pub fn reconnect(&mut self) -> serial::Result<()> {
        self.port = serial::open(&self.port_name)?;
        Ok(())
    }

    // /// Sets the internal antenna tuner status.
    // ///
    // /// p1: 0 = RX-AT THRU; 1 = RX-AT IN
    // ///
    // /// p2: 0 = TX-AT THRU; 1 = TX-AT IN
    // ///
    // /// p3: 0 = Stop tuning; 1 = Start tuning
    // pub fn set_tuner_status(&mut self, p1: u8, p2: u8, p3: u8) -> RadioResult<()> {
    //     self.transmit(&format!("AC{}{}{};", p1, p2, p3))?;
    //     Ok(Ok(()))
    // }

    /// Selects the antenna connector ANT1/ANT2
    ///
    /// p1: 0 = ANT1; 1 = ANT2
    pub fn set_antenna(&mut self, p1: u8) -> RadioResult<()> {
        Ok(self.transmit(&format!("AN{};", p1))?)
    }

    // pub fn read_antenna(&mut self) -> RadioResult<u8> {
    //     let _ = self.transmit("AN;")?;
    //     let data = self.receive()?;
    // }

    /// Moves down the frequency band
    pub fn frequency_down(&mut self) -> RadioResult<()> {
        self.transmit("BD;")
    }

    /// Moves up the frequency band
    pub fn frequency_up(&mut self) -> RadioResult<()> {
        self.transmit("BU;")
    }

    /// Attempts to receive data from the radio. Currently, this
    /// blocks indefinitely until the serial port's CTS pin goes true.
    pub fn receive(&mut self) -> RadioResult<AsciiString> {
        let mut buf = Vec::new();
        self.port.set_rts(false)?;
        // while ! self.port.read_cts()? {}
        self.port.read_to_end(&mut buf)?;

        let mut ascii = AsciiString::new();
        for num in buf {
            if let Ok(ascii_char) = num.to_ascii_char() {
                ascii.push(ascii_char);
            }
        }

        Ok(Ok(ascii))
    }

    pub fn transmit(&mut self, data: &str) -> RadioResult<()> {
        self.port.set_rts(true)?;
        self.port.write(data.as_bytes())?;
        Ok(Ok(()))
    }

    #[allow(dead_code)]
    fn check_for_error(e: &str) -> Option<RadioError> {
        match e {
            "?;" => Some(RadioError::SyntaxOrStatus),
            "E;" => Some(RadioError::CommError),
            "O;" => Some(RadioError::ProcIncomplete),
            _ => None,
        }
    }
}

impl Drop for TS480 {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        self.port.set_rts(false);
        self.port.set_dtr(true);
    }
}

pub enum RadioError {
    /// `?;` response from the radio.
    ///
    /// Indicates either the command syntax was incorrect or
    /// the command was not executed due to the tranceiver's current status
    SyntaxOrStatus,

    /// `E;` response from the radio.
    ///
    /// Indicates a communcation error.
    CommError,

    /// `O;` response from the radio.
    ///
    /// Indicates receive data was sent but
    /// processing was not completed.
    ProcIncomplete,
}
