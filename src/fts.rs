#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FTS(pub u64);

impl FTS {
    pub fn from_le_bytes(bytes: [u8; 8]) -> Self {
        FTS(u64::from_le_bytes(bytes))
    }

    pub fn as_secs_f64(&self) -> f64 {
        self.0 as f64 / 1000.0
    }

    pub fn shift(&self, delta_ms: i64) -> Self {
        let shifted = (self.0 as i64 + delta_ms).max(0) as u64;
        FTS(shifted)
    }

    pub fn iso_format(&self) -> String {
        let total_ms = self.0;
        let secs = total_ms / 1000;
        let ms = total_ms % 1000;
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        let s = secs % 60;
        format!("{:02}:{:02}:{:02}.{:03}", h, m, s, ms)
    }
}
