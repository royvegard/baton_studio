use baton_studio::*;
use nusb::MaybeFuture;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let device = nusb::list_devices()
        .wait()?
        .find(|dev| dev.vendor_id() == 0x194f && dev.product_id() == 0x010d)
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "device not found",
        ))?
        .open()
        .wait()?;

    let mut command = Command::new();

    // Set all stereo bus faders to unity gain
    for m in 0..9 {
        command.set_output_fader(m, Value::Unity).send(&device)?;
    }

    // Set:
    // Daw 1 -> Line out 1, Daw 2 -> Line out 2
    // Daw 3 -> Line out 3, Daw 4 -> Line out 4
    // Daw 5 -> Line out 5, Daw 6 -> Line out 6
    // Daw 7 -> Line out 7, Daw 8 -> Line out 8
    // Daw 9 -> SPDIF out 1, Daw 10 -> SPDIF out 2
    // Daw 11 -> ADAT out 1, Daw 12 -> ADAT out 2
    // Daw 13 -> ADAT out 3, Daw 14 -> ADAT out 4
    // Daw 15 -> ADAT out 5, Daw 16 -> ADAT out 6
    // Daw 17 -> ADAT out 7, Daw 18 -> ADAT out 8
    // Everything else muted

    let mut daw_channel_left = 16;
    let mut daw_channel_right;

    for m in 0..9 {
        daw_channel_left += 2;
        daw_channel_right = daw_channel_left + 1;
        for c in 0..35 {
            if c == daw_channel_left {
                command
                    .set_input_fader(c, m, Channel::Left, Value::Unity)
                    .send(&device)?;
                command
                    .set_input_fader(c, m, Channel::Right, Value::Muted)
                    .send(&device)?;
            } else if c == daw_channel_right {
                command
                    .set_input_fader(c, m, Channel::Left, Value::Muted)
                    .send(&device)?;
                command
                    .set_input_fader(c, m, Channel::Right, Value::Unity)
                    .send(&device)?;
            } else {
                command
                    .set_input_fader(c, m, Channel::Left, Value::Muted)
                    .send(&device)?;
                command
                    .set_input_fader(c, m, Channel::Right, Value::Muted)
                    .send(&device)?;
            }
        }
    }

    Ok(())
}
