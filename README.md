# baton-studio

[![Crates.io](https://img.shields.io/crates/v/baton-studio.svg)](https://crates.io/crates/baton-studio)
[![Documentation](https://docs.rs/baton-studio/badge.svg)](https://docs.rs/baton-studio)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust library for controlling the Presonus STUDIO1824c USB audio interface.

## Features

The Presonus STUDIO1824c is a professional USB audio interface that features:

- **18 input channels:**
  - 8 analog input channels with mic/line/instrument preamps
  - 2 digital S/PDIF input channels
  - 8 digital ADAT input channels

- **18 DAW channels** (output channels from the computer's perspective)

- **24 output channels:**
  - 8 analog line output channels
  - 2 stereo headphone outputs (4 channels total)
  - 2 Main output channels
  - 2 digital S/PDIF output channels
  - 8 digital ADAT output channels

- **Physical controls:**
  - 48V phantom power button for mic inputs
  - Main output mute button
  - Main output mono button
  - Instrument/line level switch for channels 1 and 2

## Mixer Architecture

The 1824c contains 9 separate internal mixes, each with stereo fader controls for all 36 input channels (18 physical inputs + 18 DAW channels):

- **Mix 1:** Main outputs (channels 1-2) + headphone output 1
- **Mix 2:** Line outputs 3-4 + headphone output 2
- **Mix 3:** Line outputs 5-6
- **Mix 4:** Line outputs 7-8
- **Mix 5:** S/PDIF digital outputs
- **Mix 6-9:** ADAT digital outputs (pairs 1-2, 3-4, 5-6, 7-8)

This library allows you to control:
- **72 input faders** (36 channels × 2 for left/right)
- **9 stereo output faders** (one per mix)
- **4 front panel buttons**

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
baton-studio = "0.1.0"
```

## Quick Start

```rust
use baton_studio::*;
use nusb::MaybeFuture;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Find and open the device
    let device = nusb::list_devices()
        .wait()?
        .find(|dev| dev.vendor_id() == 0x194f && dev.product_id() == 0x010d)
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "STUDIO1824c not found",
        ))?
        .open()
        .wait()?;

    let mut command = Command::new();
    let mut state = State::new();

    // Enable phantom power
    command.set_button(Button::Phantom, true).send(&device)?;

    // Set input fader for channel 1 to -3dB
    command
        .set_input_fader(0, 0, Channel::Left, Value::DB(-3.0))
        .send(&device)?;

    // Set main output fader to -2.4dB
    command.set_output_fader(0, Value::DB(-2.4)).send(&device)?;

    // Poll current state and read meter values
    state.poll(&device)?;
    let mic_level = gain_to_db(state.mic[0]);
    println!("Mic 1 level: {:.1} dBFS", mic_level);

    Ok(())
}
```

## API Overview

### Commands

The `Command` struct allows you to control the device:

```rust
let mut command = Command::new();

// Control input faders (channel, mix, left/right, value)
command.set_input_fader(0, 0, Channel::Left, Value::DB(-6.0)).send(&device)?;
command.set_input_fader(0, 0, Channel::Right, Value::Unity).send(&device)?;

// Control output faders (mix, value)
command.set_output_fader(0, Value::DB(-3.0)).send(&device)?;
command.set_output_fader(1, Value::Muted).send(&device)?;

// Control buttons
command.set_button(Button::Phantom, true).send(&device)?;
command.set_button(Button::Mute, false).send(&device)?;
```

### State Monitoring

The `State` struct provides access to meter readings and button states:

```rust
let mut state = State::new();
state.poll(&device)?;

// Read input meters (dBFS values)
println!("Mic 1: {:.1} dBFS", gain_to_db(state.mic[0]));
println!("Line 1: {:.1} dBFS", gain_to_db(state.line[0]));
println!("DAW 1: {:.1} dBFS", gain_to_db(state.daw[0]));

// Read output meters
println!("Main L: {:.1} dBFS", gain_to_db(state.bus[0]));

// Read button states
println!("Phantom power: {}", state.phantom);
println!("Mute: {}", state.mute);
```

## Signal Flow

The signal routing works as follows:

```text
          +-->Fader left--->\                /-->Line Output 1
Input 1-->|                  |Stereo fader 1|
          +-->Fader right-->/                \-->Line Output 2
          |
         ...
          |
          +-->Fader left--->\                /-->ADAT Output 7
          |                  |Stereo fader 9|
          +-->Fader right-->/                \-->ADAT Output 8
```

Each of the 36 input channels can be routed to all 9 output mixes with independent left/right fader control.

## Hardware Requirements

- Presonus STUDIO1824c USB audio interface
- USB connection to your computer
- Proper USB permissions (you may need to run as root or configure udev rules on Linux)

## Examples

See the `examples/` directory for more complete usage examples:

## Testing

The library includes comprehensive tests:

```bash
# Run all tests (requires connected STUDIO1824c)
cargo test

```

## Platform Support

This library should work on any platform supported by the `nusb` crate:
- Linux
- macOS

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- Built on top of the excellent [`nusb`](https://crates.io/crates/nusb) USB library
- Inspired by the need for programmatic control of professional audio equipment
