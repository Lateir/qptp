use serde::Serialize;

// What the qptp driver reported in its last acknowledgement. QPTP1 drivers only ever run Classic.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DriverReport { pub mode: Option<String>, pub adopted: [bool; 2], pub quest3: [bool; 2], pub conflict: Option<String> }

// Parses `QPTP1` or `QPTP2 mode=fusion left=1 right=0 q3left=0 q3right=0 conflict=none`; q3* marks controllers that report a Quest 3 model.
pub(crate) fn parse_ack(reply: &[u8]) -> Option<DriverReport> {
    if reply == b"QPTP1" { return Some(DriverReport { mode: Some("classic".into()), ..DriverReport::default() }) }
    let text = std::str::from_utf8(reply).ok()?;
    let mut words = text.split_whitespace();
    if words.next()? != "QPTP2" { return None }
    let mut report = DriverReport::default();
    for word in words {
        let Some((key, value)) = word.split_once('=') else { continue };
        match key {
            "mode" if matches!(value, "classic" | "fusion" | "fallback") => report.mode = Some(value.into()),
            "left" => report.adopted[0] = value == "1",
            "right" => report.adopted[1] = value == "1",
            "q3left" => report.quest3[0] = value == "1",
            "q3right" => report.quest3[1] = value == "1",
            "conflict" if value != "none" && !value.is_empty() => report.conflict = Some(value.chars().take(32).collect()),
            _ => {}
        }
    }
    report.mode.is_some().then_some(report)
}

#[cfg(windows)]
mod windows {
    use super::{parse_ack, DriverReport};
    use crate::{controller_live, tray_controller, Snapshot};
    use std::{net::UdpSocket, sync::{Mutex, OnceLock}, time::{Duration, Instant}};

    struct Link { socket: UdpSocket, last_ack: Option<(Instant, DriverReport)> }
    static LINK: OnceLock<Mutex<Option<Link>>> = OnceLock::new();

    fn lines(snapshot: &Snapshot, side: &str, output: &mut Vec<String>) {
        let (sensor, buttons, pad) = if side == "left" {
            (&snapshot.left, snapshot.left_buttons, snapshot.left_pad)
        } else {
            (&snapshot.right, snapshot.right_buttons, snapshot.right_pad)
        };
        // The pad values are already gated by mode and controller state in touch::Engine::step.
        let live = controller_live(snapshot, side);
        let single = live && snapshot.input_mode == "button" && buttons[0];
        let double_first = live && snapshot.input_mode == "two_buttons" && buttons[0];
        let double_second = live && snapshot.input_mode == "two_buttons" && buttons[1];
        output.push(format!("{side}_pad_x={:.5}", pad.x));
        output.push(format!("{side}_pad_y={:.5}", pad.y));
        output.push(format!("{side}_pad_touch={}", u8::from(pad.touch)));
        output.push(format!("{side}_pad_force={:.5}", pad.force));
        output.push(format!("{side}_pad_click={}", u8::from(pad.click)));
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

    pub(super) fn publish(snapshot: &Snapshot) -> (bool, DriverReport) {
        let link = LINK.get_or_init(|| Mutex::new(None));
        let Ok(mut link) = link.lock() else { return (false, DriverReport::default()) };
        if link.is_none() {
            *link = UdpSocket::bind("127.0.0.1:0").ok().and_then(|socket| {
                socket.set_nonblocking(true).ok()?;
                Some(Link { socket, last_ack: None })
            });
        }
        let Some(link) = link.as_mut() else { return (false, DriverReport::default()) };
        let mut fields = Vec::with_capacity(26);
        lines(snapshot, "left", &mut fields);
        lines(snapshot, "right", &mut fields);
        let _ = link.socket.send_to(fields.join("\n").as_bytes(), "127.0.0.1:39571");
        let mut reply = [0u8; 128];
        while let Ok((length, peer)) = link.socket.recv_from(&mut reply) {
            if !peer.ip().is_loopback() || peer.port() != 39571 { continue }
            if let Some(report) = parse_ack(&reply[..length]) { link.last_ack = Some((Instant::now(), report)) }
        }
        match &link.last_ack {
            Some((time, report)) if time.elapsed() < Duration::from_secs(2) => (true, report.clone()),
            _ => (false, DriverReport::default()),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        fn refreshed(snapshot: &mut Snapshot, side: &str) -> Vec<String> {
            crate::refresh_pads(snapshot, 0);
            let mut fields = Vec::new();
            lines(snapshot, side, &mut fields);
            fields
        }
        #[test]
        fn modes_route_only_their_own_inputs() {
            let mut snapshot = Snapshot::default();
            snapshot.phase = "connected".into();
            snapshot.left.x = 255;
            snapshot.left.y = 0;
            snapshot.left.force = 0.8;
            snapshot.left_buttons = [true, true];
            snapshot.engine.feed(0, crate::touch_raw(&snapshot.left), 0, &snapshot.touchpad);
            let fields = refreshed(&mut snapshot, "left");
            assert!(fields.iter().any(|s| s == "left_pad_x=1.00000"));
            assert!(fields.iter().any(|s| s == "left_pad_touch=1"));
            assert!(fields.iter().any(|s| s == "left_pad_click=1"));
            assert!(fields.iter().any(|s| s == "left_single_button=0"));
            snapshot.input_mode = "button".into();
            let fields = refreshed(&mut snapshot, "left");
            assert!(fields.iter().any(|s| s == "left_pad_touch=0"));
            assert!(fields.iter().any(|s| s == "left_pad_click=0"));
            assert!(fields.iter().any(|s| s == "left_single_button=1"));
            assert!(fields.iter().any(|s| s == "left_double_button_1=0"));
            assert!(fields.iter().any(|s| s == "left_double_button_2=0"));
            snapshot.input_mode = "two_buttons".into();
            let fields = refreshed(&mut snapshot, "left");
            assert!(fields.iter().any(|s| s == "left_single_button=0"));
            assert!(fields.iter().any(|s| s == "left_double_button_1=1"));
            assert!(fields.iter().any(|s| s == "left_double_button_2=1"));
            snapshot.input_mode = "touchpad".into();
            snapshot.phase = "searching".into();
            let fields = refreshed(&mut snapshot, "left");
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
            let left = refreshed(&mut snapshot, "left");
            let right = refreshed(&mut snapshot, "right");
            for expected in ["left_stylus=0.25000", "left_trigger_proximity=0.50000", "left_trigger_slide=0.75000", "left_battery_valid=1", "left_battery=0.78000"] {
                assert!(left.iter().any(|field| field == expected), "missing {expected}");
            }
            assert!(right.iter().any(|field| field == "right_stylus=0.00000"));
            assert!(right.iter().any(|field| field == "right_battery_valid=0"));
        }
    }
}

#[cfg(windows)]
pub(crate) fn publish(snapshot: &crate::Snapshot) -> (bool, DriverReport) { windows::publish(snapshot) }
#[cfg(not(windows))]
pub(crate) fn publish(_: &crate::Snapshot) -> (bool, DriverReport) { (false, DriverReport::default()) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn acknowledgements_report_the_driver_mode() {
        assert_eq!(parse_ack(b"QPTP1").unwrap().mode.as_deref(), Some("classic"));
        let fusion = parse_ack(b"QPTP2 mode=fusion left=1 right=0 q3left=0 q3right=1 conflict=none").unwrap();
        assert_eq!(fusion, DriverReport { mode: Some("fusion".into()), adopted: [true, false], quest3: [false, true], conflict: None });
        assert_eq!(parse_ack(b"QPTP2 mode=fallback conflict=qftplus").unwrap().conflict.as_deref(), Some("qftplus"));
        assert!(parse_ack(b"QPTP2 mode=other").is_none());
        assert!(parse_ack(b"hello").is_none());
    }
}
