use nusb::{
    Device, MaybeFuture,
    transfer::{ControlIn, ControlOut, ControlType, Recipient, TransferError},
};
use std::time::Duration;

// Modes
const MODE_BUTTON: u32 = 0x00;
const MODE_CHANNEL_STRIP: u32 = 0x64;
const MODE_BUS_STRIP: u32 = 0x65;

// Buttons
const BUTTON_1_2_LINE: u32 = 0x00;
const BUTTON_MAIN_MUTE: u32 = 0x01;
const BUTTON_MAIN_MONO: u32 = 0x02;
const BUTTON_PHANTOM_POWER: u32 = 0x04;

// Output channels
const LEFT: u32 = 0x00;
const RIGHT: u32 = 0x01;

// Fader presets
const MUTED: u32 = 0x00;
const CHANNEL_UNITY: u32 = 0x0100_0000;

pub struct Command {
    pub mode: u32,
    pub input_strip: u32,
    pub output_bus: u32,
    pub output_channel: u32,
    value: u32,
}

impl Command {
    pub fn new() -> Self {
        Command {
            mode: MODE_CHANNEL_STRIP,
            input_strip: 0x00,
            output_bus: 0x04,
            output_channel: LEFT,
            value: 0x00000000,
        }
    }

    fn as_array(&self) -> [u8; 28] {
        const FIX1: u32 = 0x50617269;
        const FIX2: u32 = 0x14;

        let mut arr = [0u8; 28];
        let mut i = 0;

        for b in self.mode.to_le_bytes() {
            arr[i] = b;
            i += 1;
        }
        for b in self.input_strip.to_le_bytes() {
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
        for b in self.output_channel.to_le_bytes() {
            arr[i] = b;
            i += 1;
        }
        for b in self.value.to_le_bytes() {
            arr[i] = b;
            i += 1;
        }

        arr
    }

    pub fn set_db(&mut self, db: f64) -> &Self {
        self.value = (CHANNEL_UNITY as f64 * 10.0_f64.powf(db.clamp(-96.0, 10.0) / 20.0)) as u32;
        self
    }

    pub fn set_button(&mut self, button: u32, value: bool) -> &Self {
        self.input_strip = 0x00;
        self.output_bus = 0x00;
        self.mode = MODE_BUTTON;
        self.output_channel = button;
        self.value = match value {
            true => 1,
            false => 0,
        };
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

        //block_on(device.control_out(fader_control)).into_result()
        device
            .control_out(fader_control, Duration::from_millis(100))
            .wait()
    }
}

pub struct State {
    counter: u16,
    /// Microphone input meters.
    mic: [u32; 8],
    /// S/PDIF input meters.
    spdif: [u32; 2],
    /// ADAT input meters.
    adat: [u32; 8],
    /// DAW input meters.
    daw: [u32; 18],
    /// Stereo busses meters.
    bus: [u32; 18],
    /// 48V phantom power.
    phantom: u32,
    /// Channel 1-2 line mode.
    line: u32,
    /// Main mix mute.
    mute: u32,
    /// Main mix mono.
    mono: u32,
}

impl State {
    fn new() -> Self {
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

    /// Reset all values to zero.
    /// This is used before requesting state from device.
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

    /// Convert a slice of 4 bytes to a u32.
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

    /// Convert from integer amplitude to db amplitude.
    fn get_db(input: u32) -> f64 {
        const ZERO_DBFS: u32 = 0x8000_0000;
        20.0 * (input as f64 / ZERO_DBFS as f64).log10()
    }

    fn poll(&mut self, device: &Device) -> Result<(), TransferError> {
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

        command
            .set_button(BUTTON_1_2_LINE, true)
            .send(&device)
            .unwrap();
        thread::sleep(Duration::from_secs(1));
        state.poll(&device).unwrap();
        assert_eq!(state.line, 1);

        command
            .set_button(BUTTON_1_2_LINE, false)
            .send(&device)
            .unwrap();
        thread::sleep(Duration::from_secs(1));
        state.poll(&device).unwrap();
        assert_eq!(state.line, 0);

        command
            .set_button(BUTTON_MAIN_MONO, true)
            .send(&device)
            .unwrap();
        thread::sleep(Duration::from_secs(1));
        state.poll(&device).unwrap();
        assert_eq!(state.mono, 1);

        command
            .set_button(BUTTON_MAIN_MONO, false)
            .send(&device)
            .unwrap();
        thread::sleep(Duration::from_secs(1));
        state.poll(&device).unwrap();
        assert_eq!(state.mono, 0);

        command
            .set_button(BUTTON_MAIN_MUTE, true)
            .send(&device)
            .unwrap();
        thread::sleep(Duration::from_secs(1));
        state.poll(&device).unwrap();
        assert_eq!(state.mute, 1);
        command
            .set_button(BUTTON_MAIN_MUTE, false)
            .send(&device)
            .unwrap();
        state.poll(&device).unwrap();
        assert_eq!(state.mute, 0);

        thread::sleep(Duration::from_secs(1));
        command
            .set_button(BUTTON_PHANTOM_POWER, true)
            .send(&device)
            .unwrap();
        thread::sleep(Duration::from_secs(1));
        state.poll(&device).unwrap();
        assert_eq!(state.phantom, 1);
        command
            .set_button(BUTTON_PHANTOM_POWER, false)
            .send(&device)
            .unwrap();
        state.poll(&device).unwrap();
        assert_eq!(state.phantom, 0);
    }
}
