#![warn(missing_docs)]
//! Talk to your Presonus STUDIO1824c
//!
//! This library allows you to control most of the functions
//! of the 1824c audio interface.
//!
//! The Presonus STUDIO1824c is a USB audio interface that features
//! - 18 input channels:
//!   - 8 analog input channels with mic/line/instr preamps
//!   - 2 didgital S/PDIF input channels
//!   - 8 digital ADAT input channels
//! - 18 DAW channels i.e. channels that appear as output channels from the point of view of the computer
//! - 24 output channels:
//!   - 8 analog line output channels
//!   - 2 stereo headphone outputs (total 4 channels)
//!   - 2 Main output channels
//!   - 2 digital S/PDIF output channels
//!   - 8 digital ADAT output channels
//! - Button for 48V phantom power for the 8 mic input channels
//! - Button for Main output mute
//! - Button for Main output mono
//! - Button to switch between instrument- and line-level on the 1/4 inch inputs on channel 1 and 2
//!
//! ## Mixer
//! The internals of the 1824c contains 9 separate mixes.
//! One mix contains stereo fader controls for all 18 input channels plus all 18 DAW channels for a total of 36 channels.
//! The output for mix number one is output channels 1 and 2, and at the same time Main output channels 1 and 2, and stereo headphone output number 1.
//! The output for mix number two is output channels 3 and 4, and also stereo headphone output number 2.
//! The output for mix number three is output channels 5 and 6.
//! Mix four is output channels 7 and 8.
//! Mix five is the 2 digital S/PDIF output channels.
//! Mix six through nine is the ADAT digital output channels (six -> ADAT 1,2, seven -> ADAT 3,4, etc.)
//!
//! Think of it as a mixer with 36 input channels and 9 buses.
//!
//! ## Signal flow
//! A signal flow from one input channel to all nine buses looks something like this:
//! ```text
//!           +-->Fader left--->\                /-->Line Output 1
//! Input 1-->|                  |Stereo fader 1|
//!           +-->Fader right-->/                \-->Line Output 2
//!           |
//!          ...
//!           |
//!           +-->Fader left--->\                /-->Line Output 7
//!           |                  |Stereo fader 4|
//!           +-->Fader right-->/                \-->Line Output 8
//!           |
//!           |
//!           +-->Fader left--->\                /-->S/PDIF Output left
//!           |                  |Stereo fader 5|
//!           +-->Fader right-->/                \-->S/PDIF Output right
//!           |
//!           |
//!           +-->Fader left--->\                /-->ADAT Output 1
//!           |                  |Stereo fader 6|
//!           +-->Fader right-->/                \-->ADAT Output 2
//!           |
//!          ...
//!           |
//!           +-->Fader left--->\                /-->ADAT Output 7
//!           |                  |Stereo fader 9|
//!           +-->Fader right-->/                \-->ADAT Output 8
//! ```
//! The signal flow is identical for all of the 36 input channels.
//! With this library you can control all of the Fader left and Fader right faders, and all of the Stereo faders.
//! In total you can control 36*2=72 left/right Faders and 9 Stereo faders.
//!
//! The Fader left and Fader right, or input faders are controlled with [`set_input_fader()`](Command::set_input_fader).
//! The Stereo faders, or output faders are controlled with [`set_output_fader()`](Command::set_output_fader).
//!
//! ## Buttons
//! The four buttons on the front panel of the audio interface can be controlled with [`set_button()`](Command::set_button).
//!
use nusb::{
    Device, MaybeFuture,
    transfer::{ControlIn, ControlOut, ControlType, Recipient, TransferError},
};
use std::time::Duration;

// Fader presets
/// A fader gain value that corresponds to muted.
const MUTED: u32 = 0x00;
/// A fader gain value that corresponds to unity gain.
const UNITY: u32 = 0x0100_0000;

/// Push buttons on the front panel of the Studio 1824c.
#[derive(Clone, Copy)]
pub enum Button {
    /// Switches between instrument- and line-level on
    /// the 1/4-inch inputs for channel 1 and 2.
    Line = 0x00,
    /// Mutes the Main output signal.
    Mute = 0x01,
    /// Sums the Main stereo output signal to mono.
    Mono = 0x02,
    /// 48V phantom power for all microphone inputs.
    Phantom = 0x04,
}

/// Value for faders
pub enum Value {
    /// Decibel
    DB(f64),
    /// Raw gain value
    Gain(u32),
    /// Unity gain
    Unity,
    /// Zero gain or muted
    Muted,
}

/// Output channels
#[derive(Clone, Copy)]
pub enum Channel {
    /// Left channel
    Left = 0x00,
    /// Right channel
    Right = 0x01,
}

#[derive(Clone, Copy)]
enum Mode {
    Button = 0x00,
    ChannelStrip = 0x64,
    BusStrip = 0x65,
}

/// Commands to send to the audio device.
///
/// Command is used to prepare and send commands to the
/// Presonus STUDIO1824c. A Command can be prepared to
/// set the buttons on the front panel, or to set the
/// faders.
///
/// # Examples
/// ```
/// # use std::error::Error;
/// use baton_studio::{Button, Channel, Command, Value};
/// use nusb::MaybeFuture;
///
/// # fn main() -> Result<(), Box<dyn Error>> {
/// // Open the 1824c usb device.
/// let my_1824c = nusb::list_devices()
///    .wait()?
///    .find(|dev| dev.vendor_id() == 0x194f && dev.product_id() == 0x010d)
///    .ok_or(std::io::Error::new(std::io::ErrorKind::NotFound, "device not found"))?
///    .open()
///    .wait()?;
///
/// let mut command = Command::new();
///
/// // Prepare command to set the Mute button to true.
/// command.set_button(Button::Mute, true);
/// // Send the command.
/// command.send(&my_1824c)?;
///
/// // Prepare and send command to set fader.
/// command
///     .set_input_fader(0, 0, Channel::Left, Value::Unity)
///     .send(&my_1824c)?;
///
/// # Ok(())
/// # }
/// ```
pub struct Command {
    mode: Mode,
    input_strip: u32,
    output_bus: u32,
    output_channel: Channel,
    button: Button,
    value: u32,
}

impl Default for Command {
    fn default() -> Self {
        Self::new()
    }
}

impl Command {
    /// Initialize the Command struct.
    pub fn new() -> Self {
        Command {
            mode: Mode::ChannelStrip,
            input_strip: 0x00,
            output_bus: 0x00,
            output_channel: Channel::Left,
            button: Button::Line,
            value: 0x00000000,
        }
    }

    fn as_array(&self) -> [u8; 28] {
        const FIX1: u32 = 0x50617269;
        const FIX2: u32 = 0x14;

        let mut arr = [0u8; 28];
        let mut i = 0;

        for b in (self.mode as u32).to_le_bytes() {
            arr[i] = b;
            i += 1;
        }
        let value = match self.mode {
            Mode::Button => 0x00,
            Mode::ChannelStrip => self.input_strip,
            Mode::BusStrip => self.output_bus,
        };
        for b in value.to_le_bytes() {
            arr[i] = b;
            i += 1;
        }
        for b in FIX1.to_le_bytes() {
            arr[i] = b;
            i += 1;
        }
        for b in FIX2.to_le_bytes() {
            arr[i] = b;
            i += 1;
        }
        for b in self.output_bus.to_le_bytes() {
            arr[i] = b;
            i += 1;
        }
        let value = match self.mode {
            Mode::Button => self.button as u32,
            Mode::ChannelStrip => self.output_channel as u32,
            Mode::BusStrip => Channel::Left as u32,
        };
        for b in value.to_le_bytes() {
            arr[i] = b;
            i += 1;
        }
        for b in self.value.to_le_bytes() {
            arr[i] = b;
            i += 1;
        }

        arr
    }

    /// Prepare a Button command.
    ///
    /// Used to turn on or off one of the four buttons
    /// on the front panel of the 1824c.
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// # use baton_studio::*;
    /// # use nusb::MaybeFuture;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// # let my_1824c = nusb::list_devices()
    /// #    .wait()?.find(|dev| dev.vendor_id() == 0x194f && dev.product_id() == 0x010d)
    /// #    .ok_or(std::io::Error::new(std::io::ErrorKind::NotFound, "device not found"))?
    /// #    .open().wait()?;
    /// # let mut command = Command::new();
    /// // Prepare and send command to turn off phantom power.
    /// command.set_button(Button::Phantom, false).send(&my_1824c)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_button(&mut self, button: Button, value: bool) -> &mut Self {
        self.input_strip = 0x00;
        self.output_bus = 0x00;
        self.mode = Mode::Button;
        self.button = button;
        self.value = match value {
            true => 1,
            false => 0,
        };
        self
    }

    /// Prepare a command to set an input fader.
    ///
    /// The input faders are the faders of the 36
    ///
    /// # Arguments
    /// - `input` The input channel is a number between 0 and 35.
    /// - `output` The ouput bus is a number between 0 and 8.
    /// - `channel` The channel of the output bus.
    ///   [`Left`](Channel::Left) or [`Right`](Channel::Right).
    /// - `value` The fader value.
    ///   Use the helper function [`db_to_gain()`](db_to_gain) to easily set the value in db.
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// # use baton_studio::*;
    /// # use nusb::MaybeFuture;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// # let my_1824c = nusb::list_devices()
    /// #    .wait()?.find(|dev| dev.vendor_id() == 0x194f && dev.product_id() == 0x010d)
    /// #    .ok_or(std::io::Error::new(std::io::ErrorKind::NotFound, "device not found"))?
    /// #    .open().wait()?;
    /// # let mut command = Command::new();
    /// // Prepare and send command to set the fader of the first
    /// // input channel of the left channel of the first stereo
    /// // mix to unity gain.
    /// command.set_input_fader(0, 0, Channel::Left, Value::Unity).send(&my_1824c)?;
    ///
    /// // Set value to zero i.e. muted.
    /// command.set_input_fader(0, 0, Channel::Left, Value::Muted).send(&my_1824c)?;
    ///
    /// // Use the helper function db_to_gain()
    /// command.set_input_fader(0, 0, Channel::Left, Value::DB(-6.0)).send(&my_1824c)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_input_fader(
        &mut self,
        input: u32,
        output: u32,
        channel: Channel,
        value: Value,
    ) -> &mut Self {
        let v = match value {
            Value::DB(db) => db_to_gain(db),
            Value::Gain(g) => g,
            Value::Unity => UNITY,
            Value::Muted => MUTED,
        };
        self.mode = Mode::ChannelStrip;
        self.input_strip = input.clamp(0, 35);
        self.output_bus = output.clamp(0, 8);
        self.output_channel = channel;
        self.value = v;
        self
    }

    /// Prepare a command to set an output fader.
    pub fn set_output_fader(&mut self, output: u32, value: Value) -> &mut Self {
        let v = match value {
            Value::DB(db) => db_to_gain(db),
            Value::Gain(g) => g,
            Value::Unity => UNITY,
            Value::Muted => MUTED,
        };
        self.mode = Mode::BusStrip;
        self.output_bus = output.clamp(0, 8);
        self.value = v;
        self
    }

    pub fn send(&self, device: &Device) -> Result<(), TransferError> {
        let fader_control: ControlOut = ControlOut {
            control_type: ControlType::Vendor,
            recipient: Recipient::Device,
            request: 160,
            value: 0x0000,
            index: 0,
            data: &self.as_array(),
        };

        device
            .control_out(fader_control, Duration::from_millis(100))
            .wait()
    }
}

pub struct State {
    counter: u16,
    /// Microphone input meters.
    pub mic: [u32; 8],
    /// S/PDIF input meters.
    pub spdif: [u32; 2],
    /// ADAT input meters.
    pub adat: [u32; 8],
    /// DAW input meters.
    pub daw: [u32; 18],
    /// Stereo busses meters.
    pub bus: [u32; 18],
    /// 48V phantom power.
    pub phantom: u32,
    /// Channel 1-2 line mode.
    pub line: u32,
    /// Main mix mute.
    pub mute: u32,
    /// Main mix mono.
    pub mono: u32,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub fn new() -> Self {
        State {
            counter: 0x01,
            mic: [0x00; 8],
            spdif: [0x00; 2],
            adat: [0x00; 8],
            daw: [0x00; 18],
            bus: [0x00; 18],
            phantom: 0x00,
            line: 0x00,
            mute: 0x00,
            mono: 0x00,
        }
    }

    // Reset all values to zero.
    // This is used before requesting state from device.
    fn reset(&mut self) {
        self.mic = [0x00; 8];
        self.spdif = [0x00; 2];
        self.adat = [0x00; 8];
        self.daw = [0x00; 18];
        self.bus = [0x00; 18];
        self.phantom = 0x00;
        self.line = 0x00;
        self.mute = 0x00;
        self.mono = 0x00;
    }

    /// Return the state as an array of bytes.
    fn as_array(&self) -> [u8; 252] {
        const FIX1: u32 = 0x64656d73;
        const FIX2: u32 = 0xf4;
        const ZERO: u32 = 0x00;

        let mut arr = [0u8; 252];
        let mut i = 0;

        for b in ZERO.to_le_bytes() {
            arr[i] = b;
            i += 1;
        }
        for b in ZERO.to_le_bytes() {
            arr[i] = b;
            i += 1;
        }
        for b in FIX1.to_le_bytes() {
            arr[i] = b;
            i += 1;
        }
        for b in FIX2.to_le_bytes() {
            arr[i] = b;
            i += 1;
        }
        for m in self.mic {
            for b in m.to_le_bytes() {
                arr[i] = b;
                i += 1;
            }
        }
        for m in self.spdif {
            for b in m.to_le_bytes() {
                arr[i] = b;
                i += 1;
            }
        }
        for m in self.adat {
            for b in m.to_le_bytes() {
                arr[i] = b;
                i += 1;
            }
        }
        for m in self.daw {
            for b in m.to_le_bytes() {
                arr[i] = b;
                i += 1;
            }
        }
        for m in self.bus {
            for b in m.to_le_bytes() {
                arr[i] = b;
                i += 1;
            }
        }
        for b in self.phantom.to_le_bytes() {
            arr[i] = b;
            i += 1;
        }
        for b in self.line.to_le_bytes() {
            arr[i] = b;
            i += 1;
        }
        for b in self.mute.to_le_bytes() {
            arr[i] = b;
            i += 1;
        }
        for b in self.mono.to_le_bytes() {
            arr[i] = b;
            i += 1;
        }
        for b in ZERO.to_le_bytes() {
            arr[i] = b;
            i += 1;
        }

        arr
    }

    // Convert a slice of 4 bytes to a u32.
    fn slice_to_u32(slice: &[u8]) -> u32 {
        let mut out: u32 = slice[0] as u32;
        out += slice[1] as u32 * 0x100;
        out += slice[2] as u32 * 0x100 * 0x100;
        out += slice[3] as u32 * 0x100 * 0x100 * 0x100;

        out
    }

    fn parse_state(&mut self, slice: Vec<u8>) {
        const MIC_INDEX: usize = 0x10;
        const ADAT_INDEX: usize = 0x38;
        const SPDIF_INDEX: usize = 0x30;
        const DAW_INDEX: usize = 0x58;
        const BUS_INDEX: usize = 0xa0;

        for i in 0..self.mic.len() {
            self.mic[i] = Self::slice_to_u32(&slice[MIC_INDEX + 4 * i..=MIC_INDEX + 4 * i + 4]);
        }
        for i in 0..self.adat.len() {
            self.adat[i] = Self::slice_to_u32(&slice[ADAT_INDEX + 4 * i..=ADAT_INDEX + 4 * i + 4]);
        }
        for i in 0..self.spdif.len() {
            self.spdif[i] =
                Self::slice_to_u32(&slice[SPDIF_INDEX + 4 * i..=SPDIF_INDEX + 4 * i + 4]);
        }
        for i in 0..self.daw.len() {
            self.daw[i] = Self::slice_to_u32(&slice[DAW_INDEX + 4 * i..=DAW_INDEX + 4 * i + 4]);
        }
        for i in 0..self.bus.len() {
            self.bus[i] = Self::slice_to_u32(&slice[BUS_INDEX + 4 * i..=BUS_INDEX + 4 * i + 4]);
        }

        self.phantom = slice[0xe8] as u32;
        self.line = slice[0xec] as u32;
        self.mute = slice[0xf0] as u32;
        self.mono = slice[0xf4] as u32;
    }

    /// Read state from device
    ///
    /// # Examples
    /// ```
    /// # use std::error::Error;
    /// use baton_studio::{gain_to_db, State};
    /// use nusb::MaybeFuture;
    ///
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// // Open the 1824c usb device.
    /// let my_1824c = nusb::list_devices()
    ///    .wait()?
    ///    .find(|dev| dev.vendor_id() == 0x194f && dev.product_id() == 0x010d)
    ///    .ok_or(std::io::Error::new(std::io::ErrorKind::NotFound, "device not found"))?
    ///    .open()
    ///    .wait()?;
    ///
    /// let mut state = State::new();
    /// state.poll(&my_1824c)?;
    ///
    /// let fader_value = gain_to_db(state.mic[0]);
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn poll(&mut self, device: &Device) -> Result<(), TransferError> {
        self.reset();

        let control: ControlOut = ControlOut {
            control_type: ControlType::Vendor,
            recipient: Recipient::Device,
            request: 161,
            value: self.counter.to_le(),
            index: 0,
            data: &self.as_array(),
        };

        device
            .control_out(control, Duration::from_millis(100))
            .wait()?;

        let control: ControlIn = ControlIn {
            control_type: ControlType::Vendor,
            recipient: Recipient::Device,
            request: 162,
            value: self.counter.to_le(),
            index: 0,
            length: self.as_array().len() as u16,
        };

        if self.counter == 0xffff {
            self.counter = 0x00;
        }
        self.counter += 1;

        self.parse_state(
            device
                .control_in(control, Duration::from_millis(100))
                .wait()?,
        );
        Ok(())
    }
}

/// Convert from dB to integer gain
pub fn db_to_gain(db: f64) -> u32 {
    (UNITY as f64 * 10.0_f64.powf(db.clamp(-120.0, 10.0) / 20.0)) as u32
}

/// Convert from integer gain to dB
pub fn gain_to_db(input: u32) -> f64 {
    const ZERO_DBFS: u32 = 0x8000_0000;
    20.0 * (input as f64 / ZERO_DBFS as f64).log10()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn open_device() -> Device {
        nusb::list_devices()
            .wait()
            .unwrap()
            .find(|dev| dev.vendor_id() == 0x194f && dev.product_id() == 0x010d)
            .unwrap()
            .open()
            .wait()
            .unwrap()
    }

    #[test]
    fn buttons() {
        let device = open_device();
        let mut command = Command::new();
        let mut state = State::new();
        let pause = Duration::from_millis(500);

        command
            .set_button(Button::Line, true)
            .send(&device)
            .unwrap();
        thread::sleep(pause);
        state.poll(&device).unwrap();
        assert_eq!(state.line, 1);

        command
            .set_button(Button::Line, false)
            .send(&device)
            .unwrap();
        thread::sleep(pause);
        state.poll(&device).unwrap();
        assert_eq!(state.line, 0);

        command
            .set_button(Button::Mono, true)
            .send(&device)
            .unwrap();
        thread::sleep(pause);
        state.poll(&device).unwrap();
        assert_eq!(state.mono, 1);

        command
            .set_button(Button::Mono, false)
            .send(&device)
            .unwrap();
        thread::sleep(pause);
        state.poll(&device).unwrap();
        assert_eq!(state.mono, 0);

        command
            .set_button(Button::Mute, true)
            .send(&device)
            .unwrap();
        thread::sleep(pause);
        state.poll(&device).unwrap();
        assert_eq!(state.mute, 1);
        command
            .set_button(Button::Mute, false)
            .send(&device)
            .unwrap();
        state.poll(&device).unwrap();
        assert_eq!(state.mute, 0);

        thread::sleep(pause);
        command
            .set_button(Button::Phantom, true)
            .send(&device)
            .unwrap();
        thread::sleep(pause);
        state.poll(&device).unwrap();
        assert_eq!(state.phantom, 1);
        command
            .set_button(Button::Phantom, false)
            .send(&device)
            .unwrap();
        state.poll(&device).unwrap();
        assert_eq!(state.phantom, 0);
    }

    #[test]
    fn fader() {
        // This test needs a stable audio signal connected to Daw1
        // Uniform white noise at -18dBFS should work.
        let device = open_device();
        let mut command = Command::new();
        let mut state = State::new();

        // Mute all inputs
        for c in 0..36 {
            command
                .set_input_fader(c, 0, Channel::Left, Value::Muted)
                .send(&device)
                .unwrap();
            command
                .set_input_fader(c, 0, Channel::Right, Value::Muted)
                .send(&device)
                .unwrap();
        }

        let samples = 10;
        let pause = Duration::from_millis(123);

        let mut test_procedure = |attenuation_in, attenuation_out| {
            command
                .set_input_fader(18, 0, Channel::Left, Value::DB(attenuation_in))
                .send(&device)
                .unwrap();
            command
                .set_output_fader(0, Value::DB(attenuation_out))
                .send(&device)
                .unwrap();
            let mut sum_out = 0.0;
            let mut sum_in = 0.0;

            state.poll(&device).unwrap();
            for s in 1..=samples * 6 {
                thread::sleep(pause);
                state.poll(&device).unwrap();
                if s > samples * 2 && s <= samples * 3 {
                    sum_out += gain_to_db(state.bus[0]);
                    sum_in += gain_to_db(state.daw[0]);
                }
            }
            let average_out = sum_out / samples as f64;
            let average_in = sum_in / samples as f64;
            let expected_out = average_in + attenuation_in + attenuation_out;
            println!("in:  {average_in}");
            println!("exp: {expected_out}");
            println!("out: {average_out}");
            assert!((average_in + attenuation_in - average_out + attenuation_out).abs() < 1.0);
        };

        test_procedure(-32.144, -30.8843);
        test_procedure(0.0, 0.0);
        test_procedure(-12.0, 0.0);
        test_procedure(0.0, -6.0);
        test_procedure(-6.144, -7.8843);
    }
}
