#[cfg(test)]
mod tests {
    use baton_studio::*;
    use nusb::{Device, MaybeFuture};
    use std::thread;
    use std::time::Duration;

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
    #[ignore]
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
