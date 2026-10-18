// Thumb-rest touch processing: calibration, resting-thumb rejection, force click and pad styles.
// Ported from QFTPlus src/steamvr/contact.h (MIT, QproFaceTracking contributors).
use serde::{Deserialize, Serialize};
use std::{sync::OnceLock, time::Instant};

const DECIDE_US: u64 = 50_000; // An undecided contact becomes active after this long.
const FLAT_US: u64 = 60_000; // An active contact that stays flattened this long is a resting thumb.
const STALE_US: u64 = 500_000; // The module repeats unchanged frames every 195 ms.
const RESTING_SIZE: f32 = 75.0; const FLAT_MARGIN: f32 = 10.0;
const SWIPE_DECAY_US: f32 = 350_000.0; const SWIPE_DEADZONE: f32 = 0.3; const GLIDE_US: u64 = 2_000_000;
const RAIL_ANGLE: f32 = 35.0; const RAIL_START: f32 = 0.4; const RAIL_RELEASE: f32 = 0.8;
pub const STYLES: [&str; 4] = ["absolute", "joystick", "swipe", "mouse"];

pub fn now_us() -> u64 { static START: OnceLock<Instant> = OnceLock::new(); START.get_or_init(Instant::now).elapsed().as_micros() as u64 }

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Calibration { pub x_min: f32, pub x_max: f32, pub y_min: f32, pub y_max: f32, pub force_low: f32, pub force_high: f32, pub rotation_left: f32, pub rotation_right: f32 }
impl Default for Calibration {
    // Identity: an uncalibrated pad reports exactly what earlier versions did.
    fn default() -> Self { Self { x_min: 0.0, x_max: 255.0, y_min: 0.0, y_max: 255.0, force_low: 0.0, force_high: 1.0, rotation_left: 0.0, rotation_right: 0.0 } }
}
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Settings { pub style: String, pub click_press: f32, pub click_release: f32, pub joystick_range: f32, pub swipe_gain: f32, pub mouse_speed: f32, pub palm_rejection: bool, pub calibration: Calibration }
impl Default for Settings {
    fn default() -> Self { Self { style: "absolute".into(), click_press: 0.70, click_release: 0.60, joystick_range: 0.5, swipe_gain: 0.35, mouse_speed: 1200.0, palm_rejection: true, calibration: Calibration::default() } }
}
fn round(value: f32, step: f32) -> f32 { (value / step).round() * step }
fn within(value: f32, lo: f32, hi: f32, fallback: f32) -> f32 { if value.is_finite() { value.clamp(lo, hi) } else { fallback } }
impl Settings {
    // Repairs a loaded config: every field falls back to its default when it is out of shape.
    pub fn sanitize(&mut self) {
        let d = Settings::default(); let c = Calibration::default();
        if !STYLES.contains(&self.style.as_str()) { self.style = d.style }
        self.click_press = within(self.click_press, 0.05, 1.0, d.click_press);
        self.click_release = within(self.click_release, 0.0, self.click_press, d.click_release.min(self.click_press));
        self.joystick_range = within(self.joystick_range, 0.15, 1.0, d.joystick_range);
        self.swipe_gain = within(self.swipe_gain, 0.05, 1.0, d.swipe_gain);
        self.mouse_speed = within(self.mouse_speed, 200.0, 4000.0, d.mouse_speed);
        let k = &mut self.calibration;
        let finite = [k.x_min, k.x_max, k.y_min, k.y_max, k.force_low, k.force_high, k.rotation_left, k.rotation_right].iter().all(|v| v.is_finite());
        if !finite { *k = c; return }
        if k.x_min < 0.0 || k.x_max > 255.0 || k.x_max - k.x_min < 32.0 { (k.x_min, k.x_max) = (c.x_min, c.x_max) }
        if k.y_min < 0.0 || k.y_max > 255.0 || k.y_max - k.y_min < 32.0 { (k.y_min, k.y_max) = (c.y_min, c.y_max) }
        if k.force_low < 0.0 || k.force_high > 1.0 || k.force_high - k.force_low < 0.05 { (k.force_low, k.force_high) = (c.force_low, c.force_high) }
        k.rotation_left = k.rotation_left.clamp(-60.0, 60.0); k.rotation_right = k.rotation_right.clamp(-60.0, 60.0);
    }
    // Validates settings coming from the UI and rounds them the way they are stored.
    pub fn checked(mut self) -> Result<Self, String> {
        let k = &self.calibration;
        let values = [self.click_press, self.click_release, self.joystick_range, self.swipe_gain, self.mouse_speed, k.x_min, k.x_max, k.y_min, k.y_max, k.force_low, k.force_high, k.rotation_left, k.rotation_right];
        if !STYLES.contains(&self.style.as_str()) { return Err("Неизвестный стиль тачпада".into()) }
        if values.iter().any(|v| !v.is_finite()) || self.click_release > self.click_press { return Err("Неверные настройки тачпада".into()) }
        (self.click_press, self.click_release) = (round(self.click_press, 0.01), round(self.click_release, 0.01));
        (self.joystick_range, self.swipe_gain, self.mouse_speed) = (round(self.joystick_range, 0.01), round(self.swipe_gain, 0.01), round(self.mouse_speed, 10.0));
        let k = &mut self.calibration;
        (k.rotation_left, k.rotation_right) = (round(k.rotation_left, 0.5), round(k.rotation_right, 0.5));
        (k.force_low, k.force_high) = (round(k.force_low, 0.01), round(k.force_high, 0.01));
        self.sanitize(); Ok(self)
    }
}

fn rotate(degrees: f32, x: f32, y: f32) -> (f32, f32) {
    let (s, c) = degrees.to_radians().sin_cos();
    ((c * x - s * y).clamp(-1.0, 1.0), (s * x + c * y).clamp(-1.0, 1.0))
}
// Raw x and decoded y (0 is the top) to -1..1 with +y up, then rotated to the thumb's axis.
pub fn normalize(k: &Calibration, side: usize, x: u16, y: u16) -> (f32, f32) {
    let nx = ((x as f32 - k.x_min) / (k.x_max - k.x_min) * 2.0 - 1.0).clamp(-1.0, 1.0);
    let ny = (1.0 - (y as f32 - k.y_min) / (k.y_max - k.y_min) * 2.0).clamp(-1.0, 1.0);
    rotate(if side == 0 { k.rotation_left } else { k.rotation_right }, nx, ny)
}
pub fn force_curve(raw: f32, k: &Calibration) -> f32 { if raw.is_finite() { ((raw - k.force_low) / (k.force_high - k.force_low)).clamp(0.0, 1.0) } else { 0.0 } }

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Contact { #[default] None, Pending, Active, Rejected }
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Rail { #[default] Undecided, Vertical, Horizontal, Free }
fn rail_for(dx: f32, dy: f32) -> Rail {
    let angle = dy.atan2(dx).abs().to_degrees(); let horizontal = angle.min(180.0 - angle);
    if horizontal <= RAIL_ANGLE { Rail::Horizontal } else if 90.0 - horizontal <= RAIL_ANGLE { Rail::Vertical } else { Rail::Free }
}
fn glide(velocity: f32, touching: bool, lifted: u64, now: u64, k: &Settings) -> f32 {
    if velocity.abs() < SWIPE_DEADZONE { return 0.0 }
    let fade = if touching { 1.0 } else { (-(now.saturating_sub(lifted) as f32) / SWIPE_DECAY_US).exp() };
    let value = (velocity * k.swipe_gain * fade).clamp(-1.0, 1.0);
    if value.abs() < 0.02 { 0.0 } else { value }
}

// One raw module sample. `touch` is the module's contact flag (QPR4); `size` is the contact area (QPR4).
#[derive(Clone, Copy, Debug, Default)]
pub struct Raw { pub x: u16, pub y: u16, pub force: f32, pub size: Option<f32>, pub touch: Option<bool> }
// What SteamVR receives for one pad, plus the finger and anchor for the UI preview.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Output { pub touch: bool, pub click: bool, pub x: f32, pub y: f32, pub force: f32, pub contact: Contact, pub finger_x: f32, pub finger_y: f32, pub anchor_x: f32, pub anchor_y: f32 }

#[derive(Clone, Copy, Debug, Default)]
struct Pad { x: f32, y: f32, force: f32, down: bool, received: u64, touched: u64, flat: u64, lifted: u64, contact: Contact, anchor_x: f32, anchor_y: f32, vx: f32, vy: f32, click: bool, rail: Rail, on_path: f32, off_path: f32 }
impl Pad {
    fn begin(&mut self) { (self.anchor_x, self.anchor_y, self.rail, self.on_path, self.off_path) = (self.x, self.y, Rail::Undecided, 0.0, 0.0) }
    fn active(&self) -> bool { self.down && self.contact == Contact::Active }
    fn feed(&mut self, x: f32, y: f32, force: f32, size: Option<f32>, down: bool, now: u64, k: &Settings) {
        let last = *self; let was_active = last.active();
        (self.force, self.down, self.received) = (force, down, now);
        // A lift keeps the last position, rail and velocity so a swipe can glide on.
        if !down { if was_active { self.lifted = now } (self.contact, self.click, self.flat) = (Contact::None, false, 0); return }
        (self.x, self.y) = (x, y);
        if !last.down { (self.contact, self.touched, self.vx, self.vy, self.flat, self.click) = (Contact::Pending, now, 0.0, 0.0, 0, false) }
        match size.filter(|v| v.is_finite() && k.palm_rejection) {
            None => self.contact = Contact::Active,
            Some(_) if force >= k.click_press => self.contact = Contact::Active,
            Some(size) if self.contact == Contact::Pending && size >= RESTING_SIZE => self.contact = Contact::Rejected,
            Some(size) if self.contact == Contact::Active => {
                if size < RESTING_SIZE + FLAT_MARGIN { self.flat = 0 } else if self.flat == 0 { self.flat = now.max(1) } else if now - self.flat >= FLAT_US { self.contact = Contact::Rejected }
            }
            _ => {}
        }
        let active = self.contact == Contact::Active;
        if active && !was_active { self.begin() } else if active && now > last.received {
            self.steer(&last);
            let dt = (now - last.received) as f32 / 1e6; let blend = (dt / 0.04).min(1.0);
            self.vx += ((self.x - last.x) / 2.0 / dt - self.vx) * blend;
            self.vy += ((self.y - last.y) / 2.0 / dt - self.vy) * blend;
        }
        if !active { (self.vx, self.vy) = (0.0, 0.0) }
        self.click = active && force >= if self.click { k.click_release } else { k.click_press };
    }
    fn steer(&mut self, last: &Pad) {
        match self.rail {
            Rail::Undecided => { let (dx, dy) = (self.x - self.anchor_x, self.y - self.anchor_y); if dx.hypot(dy) >= RAIL_START { self.rail = rail_for(dx, dy) } }
            Rail::Free => {}
            rail => {
                let vertical = rail == Rail::Vertical; let (dx, dy) = ((self.x - last.x).abs(), (self.y - last.y).abs());
                self.on_path += if vertical { dy } else { dx }; self.off_path += if vertical { dx } else { dy };
                if self.off_path > RAIL_START && self.off_path > RAIL_RELEASE * self.on_path { self.rail = Rail::Free }
            }
        }
    }
    fn on_rail(&self, x: &mut f32, y: &mut f32) {
        let mut rail = self.rail;
        if rail == Rail::Undecided { let (dx, dy) = (self.x - self.anchor_x, self.y - self.anchor_y); if dx == 0.0 && dy == 0.0 { return } rail = rail_for(dx, dy) }
        if rail == Rail::Vertical { *x = 0.0 } if rail == Rail::Horizontal { *y = 0.0 }
    }
    fn touching(&mut self, now: u64) -> bool {
        if self.down && now.saturating_sub(self.received) >= STALE_US { if self.contact == Contact::Active { self.lifted = now } (self.down, self.contact, self.click) = (false, Contact::None, false) }
        if self.down && self.contact == Contact::Pending && now.saturating_sub(self.touched) >= DECIDE_US { self.contact = Contact::Active; self.begin() }
        self.active()
    }
    fn evaluate(&mut self, now: u64, k: &Settings) -> Output {
        let touching = self.touching(now);
        let mut o = Output { contact: if self.down { self.contact } else { Contact::None }, finger_x: self.x, finger_y: self.y, anchor_x: self.anchor_x, anchor_y: self.anchor_y, ..Output::default() };
        if k.style == "mouse" { return o } // The Windows cursor takes the pad; SteamVR gets nothing.
        o.force = if touching { self.force } else { 0.0 }; o.click = touching && self.click;
        match k.style.as_str() {
            "joystick" => {
                o.touch = touching;
                if touching { o.x = ((self.x - self.anchor_x) / k.joystick_range).clamp(-1.0, 1.0); o.y = ((self.y - self.anchor_y) / k.joystick_range).clamp(-1.0, 1.0); self.on_rail(&mut o.x, &mut o.y) }
            }
            "swipe" => {
                o.x = glide(self.vx, touching, self.lifted, now, k); o.y = glide(self.vy, touching, self.lifted, now, k);
                self.on_rail(&mut o.x, &mut o.y); o.touch = touching || o.x != 0.0 || o.y != 0.0;
            }
            _ => { o.touch = touching; if touching { (o.x, o.y) = (self.x, self.y) } }
        }
        o
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Cursor { tracking: bool, down: bool, x: f32, y: f32, carry_x: f32, carry_y: f32 }
impl Cursor {
    fn release(&mut self) { self.tracking = false; if self.down { mouse::button(false) } self.down = false }
    fn follow(&mut self, pad: &Pad, active: bool, speed: f32) {
        if !active { self.release(); return }
        if self.tracking {
            self.carry_x += (pad.x - self.x) / 2.0 * speed; self.carry_y -= (pad.y - self.y) / 2.0 * speed;
            let (dx, dy) = (self.carry_x.trunc(), self.carry_y.trunc()); self.carry_x -= dx; self.carry_y -= dy;
            if dx != 0.0 || dy != 0.0 { mouse::moved(dx as i32, dy as i32) }
        }
        (self.tracking, self.x, self.y) = (true, pad.x, pad.y);
        if pad.click != self.down { mouse::button(pad.click) }
        self.down = pad.click;
    }
}
#[cfg(all(windows, not(test)))]
mod mouse {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSEINPUT};
    fn send(flags: u32, dx: i32, dy: i32) {
        let input = INPUT { r#type: INPUT_MOUSE, Anonymous: INPUT_0 { mi: MOUSEINPUT { dx, dy, mouseData: 0, dwFlags: flags, time: 0, dwExtraInfo: 0 } } };
        unsafe { SendInput(1, &input, std::mem::size_of::<INPUT>() as i32) };
    }
    pub fn moved(dx: i32, dy: i32) { send(MOUSEEVENTF_MOVE, dx, dy) }
    pub fn button(down: bool) { send(if down { MOUSEEVENTF_LEFTDOWN } else { MOUSEEVENTF_LEFTUP }, 0, 0) }
}
#[cfg(not(all(windows, not(test))))]
mod mouse { pub fn moved(_: i32, _: i32) {} pub fn button(_: bool) {} }

// Both pads and their mouse cursors. Lives in the snapshot, so every change happens under its lock.
#[derive(Clone, Debug, Default)]
pub struct Engine { pads: [Pad; 2], cursors: [Cursor; 2], pub last_sent: u64 }
impl Engine {
    pub fn feed(&mut self, side: usize, raw: Raw, now: u64, k: &Settings) {
        // Without the module's contact flag, raw (0, 0) is no touch; the decoded Y axis is flipped, so idle is (0, 255).
        let down = raw.touch.unwrap_or(!(raw.x == 0 && raw.y == 255));
        let (x, y) = normalize(&k.calibration, side, raw.x, raw.y);
        self.pads[side].feed(x, y, force_curve(raw.force, &k.calibration), raw.size, down, now, k);
    }
    pub fn step(&mut self, side: usize, now: u64, k: &Settings, enabled: bool) -> Output {
        if !enabled { self.pads[side] = Pad::default(); self.cursors[side].release(); return Output::default() }
        let output = self.pads[side].evaluate(now, k);
        let pad = self.pads[side]; self.cursors[side].follow(&pad, k.style == "mouse" && pad.active(), k.mouse_speed);
        output
    }
    // True while time alone changes the output: an undecided contact or a swipe still gliding.
    pub fn busy(&self, now: u64, k: &Settings) -> bool {
        self.pads.iter().any(|p| (p.down && p.contact == Contact::Pending) || (k.style == "swipe" && !p.active() && (p.vx != 0.0 || p.vy != 0.0) && now.saturating_sub(p.lifted) < GLIDE_US))
    }
    pub fn release(&mut self) { for cursor in &mut self.cursors { cursor.release() } }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn settings(style: &str) -> Settings { Settings { style: style.into(), ..Settings::default() } }
    // A touch at decoded pad coordinates with an optional contact size.
    fn raw(x: u16, y: u16, force: f32, size: Option<f32>) -> Raw { Raw { x, y, force, size, touch: Some(true) } }
    const LIFT: Raw = Raw { x: 0, y: 255, force: 0.0, size: None, touch: Some(false) };
    #[test]
    fn identity_calibration_matches_previous_mapping_and_rotation_turns_the_axes() {
        let k = Calibration::default();
        assert_eq!(normalize(&k, 0, 255, 0), (1.0, 1.0));
        assert_eq!(normalize(&k, 0, 0, 255), (-1.0, -1.0));
        let rotated = Calibration { rotation_right: 90.0, ..k };
        let (x, y) = normalize(&rotated, 1, 255, 128);
        assert!(x.abs() < 0.01 && y > 0.99, "{x} {y}");
        let narrow = Calibration { x_min: 50.0, x_max: 200.0, force_low: 0.5, force_high: 1.0, ..k };
        assert_eq!(normalize(&narrow, 0, 200, 0).0, 1.0);
        assert_eq!(normalize(&narrow, 0, 20, 0).0, -1.0);
        assert_eq!(force_curve(0.75, &narrow), 0.5);
        assert_eq!(force_curve(0.2, &narrow), 0.0);
    }
    #[test]
    fn resting_thumb_is_rejected_by_size_and_a_firm_press_wins() {
        let k = settings("absolute"); let mut e = Engine::default();
        e.feed(0, raw(128, 128, 0.2, Some(90.0)), 0, &k);
        assert_eq!(e.step(0, 0, &k, true).contact, Contact::Rejected);
        assert!(!e.step(0, 100_000, &k, true).touch);
        e.feed(0, raw(128, 128, 0.8, Some(90.0)), 110_000, &k);
        assert!(e.step(0, 110_000, &k, true).touch);
        // A small contact waits 50 ms before it counts.
        let mut e = Engine::default();
        e.feed(0, raw(128, 128, 0.2, Some(40.0)), 0, &k);
        assert_eq!(e.step(0, 10_000, &k, true).contact, Contact::Pending);
        assert!(e.step(0, 50_000, &k, true).touch);
        // An active contact that flattens for 60 ms becomes a resting thumb.
        for (i, t) in [60_000u64, 70_000, 130_000].into_iter().enumerate() { e.feed(0, raw(128, 128, 0.2, Some(90.0)), t, &k); if i < 2 { assert!(e.step(0, t, &k, true).touch) } }
        assert_eq!(e.step(0, 130_000, &k, true).contact, Contact::Rejected);
        // Without a size (QPR3) or with rejection off, every touch is active at once.
        let mut e = Engine::default();
        e.feed(0, raw(128, 128, 0.2, None), 0, &k);
        assert!(e.step(0, 0, &k, true).touch);
        let off = Settings { palm_rejection: false, ..k }; let mut e = Engine::default();
        e.feed(0, raw(128, 128, 0.2, Some(90.0)), 0, &off);
        assert!(e.step(0, 0, &off, true).touch);
    }
    #[test]
    fn click_uses_hysteresis_and_disabled_pads_report_nothing() {
        let k = settings("absolute"); let mut e = Engine::default();
        let mut at = |force: f32, t: u64| { e.feed(1, raw(128, 128, force, None), t, &k); e.step(1, t, &k, true).click };
        assert!(!at(0.69, 0)); assert!(at(0.70, 10_000)); assert!(at(0.60, 20_000)); assert!(!at(0.59, 30_000));
        let mut e = Engine::default();
        e.feed(1, raw(255, 0, 0.9, None), 0, &k);
        assert_eq!(e.step(1, 0, &k, false), Output::default());
    }
    #[test]
    fn joystick_is_relative_to_the_touch_point_and_locks_to_a_rail() {
        let k = settings("joystick"); let mut e = Engine::default();
        e.feed(0, raw(128, 128, 0.3, None), 0, &k);
        let start = e.step(0, 0, &k, true);
        assert!(start.touch && start.x == 0.0 && start.y == 0.0);
        // Mostly upward motion locks to the vertical rail, so small sideways drift is dropped.
        e.feed(0, raw(140, 50, 0.3, None), 10_000, &k);
        let up = e.step(0, 10_000, &k, true);
        assert_eq!(up.x, 0.0); assert_eq!(up.y, 1.0);
        // Moving far sideways afterwards frees the rail.
        for (i, x) in [200u16, 250].into_iter().enumerate() { e.feed(0, raw(x, 50, 0.3, None), 20_000 + i as u64 * 10_000, &k) }
        let free = e.step(0, 30_000, &k, true);
        assert!(free.x > 0.9 && free.y > 0.9, "{free:?}");
    }
    #[test]
    fn swipe_glides_after_lift_and_fades_out() {
        let k = settings("swipe"); let mut e = Engine::default();
        for (i, y) in [200u16, 150, 100, 50].into_iter().enumerate() { e.feed(0, raw(128, y, 0.3, None), i as u64 * 10_000, &k) }
        let moving = e.step(0, 30_000, &k, true);
        assert!(moving.touch && moving.y > 0.5 && moving.x == 0.0, "{moving:?}");
        e.feed(0, LIFT, 40_000, &k);
        assert!(e.busy(40_000, &k));
        let gliding = e.step(0, 700_000, &k, true);
        assert!(gliding.touch && gliding.y > 0.0 && gliding.y < moving.y, "{gliding:?}");
        assert!(!e.step(0, 2_000_000, &k, true).touch);
        assert!(!e.busy(3_000_000, &k));
    }
    #[test]
    fn stale_touches_lift_and_mouse_style_sends_nothing_to_steamvr() {
        let k = settings("absolute"); let mut e = Engine::default();
        e.feed(0, raw(128, 128, 0.3, None), 0, &k);
        assert!(e.step(0, 400_000, &k, true).touch);
        assert!(!e.step(0, 500_000, &k, true).touch);
        let k = settings("mouse"); let mut e = Engine::default();
        e.feed(0, raw(200, 60, 0.9, None), 0, &k);
        let out = e.step(0, 0, &k, true);
        assert!(!out.touch && !out.click && out.contact == Contact::Active && out.finger_x > 0.5);
    }
    #[test]
    fn settings_are_repaired_and_validated() {
        let mut broken = Settings { style: "x".into(), click_press: f32::NAN, click_release: 2.0, joystick_range: 9.0, calibration: Calibration { x_min: 100.0, x_max: 110.0, rotation_left: 90.0, ..Calibration::default() }, ..Settings::default() };
        broken.sanitize();
        assert_eq!(broken.style, "absolute"); assert_eq!(broken.click_press, 0.70); assert_eq!(broken.click_release, 0.70); assert_eq!(broken.joystick_range, 1.0);
        assert_eq!((broken.calibration.x_min, broken.calibration.x_max), (0.0, 255.0)); assert_eq!(broken.calibration.rotation_left, 60.0);
        assert!(Settings { style: "x".into(), ..Settings::default() }.checked().is_err());
        assert!(Settings { click_press: 0.5, click_release: 0.6, ..Settings::default() }.checked().is_err());
        assert_eq!(Settings { mouse_speed: 1234.0, ..Settings::default() }.checked().unwrap().mouse_speed, 1230.0);
        let config: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(config, Settings::default());
    }
}
