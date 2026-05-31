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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_le_bytes_roundtrip() {
        let bytes = 12345u64.to_le_bytes();
        let fts = FTS::from_le_bytes(bytes);
        assert_eq!(fts.0, 12345);
    }

    #[test]
    fn as_secs_f64() {
        let fts = FTS(5500);
        assert!((fts.as_secs_f64() - 5.5).abs() < f64::EPSILON);
    }

    #[test]
    fn shift_positive() {
        let fts = FTS(1000);
        assert_eq!(fts.shift(500).0, 1500);
    }

    #[test]
    fn shift_negative() {
        let fts = FTS(1000);
        assert_eq!(fts.shift(-500).0, 500);
    }

    #[test]
    fn shift_clamps_at_zero() {
        let fts = FTS(100);
        assert_eq!(fts.shift(-200).0, 0);
    }

    #[test]
    fn iso_format() {
        assert_eq!(FTS(0).iso_format(), "00:00:00.000");
        assert_eq!(FTS(3661_123).iso_format(), "01:01:01.123");
        assert_eq!(FTS(5000).iso_format(), "00:00:05.000");
    }
}
