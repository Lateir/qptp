#[cfg(windows)]
mod windows {
    use crate::{controller_connected, tray_controller, Snapshot};
    use std::{net::UdpSocket, sync::{Mutex, OnceLock}, time::{Duration, Instant}};

    struct Link { socket: UdpSocket, last_ack: Option<Instant> }
    static LINK: OnceLock<Mutex<Option<Link>>> = OnceLock::new();

    fn active(snapshot: &Snapshot, side: &str) -> bool {
        snapshot.phase == "connected"
            && (snapshot.status.is_none() || controller_connected(snapshot, side))
    }

    fn lines(snapshot: &Snapshot, side: &str, output: &mut Vec<String>) {
        let (sensor, buttons) = if side == "left" {
            (&snapshot.left, snapshot.left_buttons)
        } else {
            (&snapshot.right, snapshot.right_buttons)
        };
        let live = active(snapshot, side);
        let touching = live && !(sensor.x == 0 && sensor.y == 255);
        let pad = snapshot.input_mode == "touchpad" && touching;
        let x = if pad { sensor.x.min(255) as f32 / 127.5 - 1.0 } else { 0.0 };
        let y = if pad { 1.0 - sensor.y.min(255) as f32 / 127.5 } else { 0.0 };
        let force = if pad && sensor.force.is_finite() { sensor.force.clamp(0.0, 1.0) } else { 0.0 };
        let single = live && snapshot.input_mode == "button" && buttons[0];
        let double_first = live && snapshot.input_mode == "two_buttons" && buttons[0];
        let double_second = live && snapshot.input_mode == "two_buttons" && buttons[1];
        output.push(format!("{side}_pad_x={x:.5}"));
        output.push(format!("{side}_pad_y={y:.5}"));
        output.push(format!("{side}_pad_touch={}", u8::from(pad)));
        output.push(format!("{side}_pad_force={force:.5}"));
        output.push(format!("{side}_single_button={}", u8::from(single)));
        output.push(format!("{side}_double_button_1={}", u8::from(double_first)));
        output.push(format!("{side}_double_button_2={}", u8::from(double_second)));
        for (name, value) in [("stylus", sensor.stylus), ("trigger_proximity", sensor.trigger_proximity), ("trigger_slide", sensor.trigger_slide)] {
            let value = if live && value.is_finite() { value.clamp(0.0, 1.0) } else { 0.0 };
            output.push(format!("{side}_{name}={value:.5}"));
        }
        let battery = tray_controller(snapshot, side).battery;
        output.push(format!("{side}_battery_valid={}", u8::from(battery.is_some())));
        output.push(format!("{side}_battery={:.5}", battery.unwrap_or(0) as f32 / 100.0));
    }

    pub(super) fn publish(snapshot: &Snapshot) -> bool {
        let link = LINK.get_or_init(|| Mutex::new(None));
        let Ok(mut link) = link.lock() else { return false };
        if link.is_none() {
            *link = UdpSocket::bind("127.0.0.1:0").ok().and_then(|socket| {
                socket.set_nonblocking(true).ok()?;
                Some(Link { socket, last_ack: None })
            });
        }
        let Some(link) = link.as_mut() else { return false };
        let mut fields = Vec::with_capacity(24);
        lines(snapshot, "left", &mut fields);
        lines(snapshot, "right", &mut fields);
        let _ = link.socket.send_to(fields.join("\n").as_bytes(), "127.0.0.1:39571");
        let mut reply = [0u8; 32];
        while let Ok((length, peer)) = link.socket.recv_from(&mut reply) {
            if peer.ip().is_loopback() && peer.port() == 39571 && &reply[..length] == b"QPTP1" {
                link.last_ack = Some(Instant::now());
            }
        }
        link.last_ack.is_some_and(|time| time.elapsed() < Duration::from_secs(2))
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn modes_route_only_their_own_inputs() {
            let mut snapshot = Snapshot::default();
            snapshot.phase = "connected".into();
            snapshot.left.x = 255;
            snapshot.left.y = 0;
            snapshot.left.force = 0.6;
            snapshot.left_buttons = [true, true];
            let mut fields = Vec::new();
            lines(&snapshot, "left", &mut fields);
            assert!(fields.iter().any(|s| s == "left_pad_x=1.00000"));
            assert!(fields.iter().any(|s| s == "left_pad_touch=1"));
            assert!(fields.iter().any(|s| s == "left_single_button=0"));
            snapshot.input_mode = "button".into();
            fields.clear();
            lines(&snapshot, "left", &mut fields);
            assert!(fields.iter().any(|s| s == "left_pad_touch=0"));
            assert!(fields.iter().any(|s| s == "left_single_button=1"));
            assert!(fields.iter().any(|s| s == "left_double_button_1=0"));
            assert!(fields.iter().any(|s| s == "left_double_button_2=0"));
            snapshot.input_mode = "two_buttons".into();
            fields.clear();
            lines(&snapshot, "left", &mut fields);
            assert!(fields.iter().any(|s| s == "left_single_button=0"));
            assert!(fields.iter().any(|s| s == "left_double_button_1=1"));
            assert!(fields.iter().any(|s| s == "left_double_button_2=1"));
            snapshot.phase = "searching".into();
            fields.clear();
            lines(&snapshot, "left", &mut fields);
            assert!(fields.iter().any(|s| s == "left_single_button=0"));
            assert!(fields.iter().any(|s| s == "left_pad_touch=0"));
        }
        #[test]
        fn analog_inputs_and_battery_follow_the_correct_side() {
            let mut snapshot = Snapshot::default();
            snapshot.phase = "connected".into();
            snapshot.status = Some(serde_json::json!({"controllers": {
                "left": {"connected": true, "battery_percent": 78},
                "right": {"connected": false, "battery_percent": 42}
            }}));
            snapshot.left.stylus = 0.25;
            snapshot.left.trigger_proximity = 0.5;
            snapshot.left.trigger_slide = 0.75;
            let mut left = Vec::new();
            let mut right = Vec::new();
            lines(&snapshot, "left", &mut left);
            lines(&snapshot, "right", &mut right);
            for expected in ["left_stylus=0.25000", "left_trigger_proximity=0.50000", "left_trigger_slide=0.75000", "left_battery_valid=1", "left_battery=0.78000"] {
                assert!(left.iter().any(|field| field == expected), "missing {expected}");
            }
            assert!(right.iter().any(|field| field == "right_stylus=0.00000"));
            assert!(right.iter().any(|field| field == "right_battery_valid=0"));
        }
    }
}

#[cfg(windows)]
pub(crate) fn publish(snapshot: &crate::Snapshot) -> bool { windows::publish(snapshot) }
#[cfg(not(windows))]
pub(crate) fn publish(_: &crate::Snapshot) -> bool { false }
