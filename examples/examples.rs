use std::error::Error;

use baton_studio::{Button, Channel, Command, State, Value, gain_to_db};
use nusb::MaybeFuture;

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
    let mut state = State::new();

    command.set_button(Button::Mute, true);
    command.send(&device).unwrap();

    command.set_button(Button::Phantom, true).send(&device)?;

    command
        .set_input_fader(0, 0, Channel::Left, Value::DB(-3.0))
        .send(&device)?;

    command
        .set_input_fader(0, 0, Channel::Left, Value::Unity)
        .send(&device)?;

    command.set_output_fader(0, Value::DB(-2.4)).send(&device)?;

    command
        .set_output_fader(1, Value::Muted)
        .send(&device)
        .unwrap();

    state.poll(&device)?;

    let mic = gain_to_db(state.mic[7]);
    println!("mic: {} dBFS", mic);

    Ok(())
}
