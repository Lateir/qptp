use std::{io, time::{Duration, Instant}};

// QPR3 may already be queued when the module receives the format request.
pub struct ContactNegotiation { request: u32, started: Instant, ready: bool }
impl ContactNegotiation {
    pub fn new(request: u32) -> Self { Self { request, started: Instant::now(), ready: false } }
    pub fn remaining(&self) -> io::Result<Duration> {
        if self.ready { return Ok(Duration::from_secs(5)) }
        Duration::from_secs(5).checked_sub(self.started.elapsed()).filter(|d| !d.is_zero())
            .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "QPR4 negotiation timed out"))
    }
    pub fn acknowledge(&mut self, id: u32, status: i32) -> io::Result<bool> {
        if id != self.request || self.ready { return Ok(false) }
        self.remaining()?;
        if status != 0 { return Err(io::Error::new(io::ErrorKind::InvalidData, "Module rejected QPR4")) }
        self.ready = true; Ok(true)
    }
    pub fn sample(&self, qpr4: bool) -> io::Result<bool> {
        self.remaining()?;
        match (self.ready, qpr4) {
            (false, false) => Ok(false),
            (true, true) => Ok(true),
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Unexpected sensor format during QPR4 negotiation")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn queued_frames_and_unrelated_haptic_ack() {
        let mut n=ContactNegotiation::new(7);
        assert!(!n.sample(false).unwrap());
        assert!(!n.acknowledge(8,0).unwrap());
        assert!(n.sample(true).is_err());
        assert!(n.acknowledge(7,0).unwrap());
        assert!(n.sample(true).unwrap());
        assert!(n.sample(false).is_err());
    }
    #[test]
    fn rejection_and_deadline_even_with_continuous_frames() {
        assert!(ContactNegotiation::new(1).acknowledge(1,1).is_err());
        let mut n=ContactNegotiation::new(1);
        n.started=Instant::now()-Duration::from_secs(6);
        assert!(n.sample(false).is_err());
        assert!(n.acknowledge(1,0).is_err());
    }
}
