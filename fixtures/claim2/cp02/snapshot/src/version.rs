/// Format metadata for the on-disk segment encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatVersion {
    pub major: u8,
    pub minor: u8,
}

impl FormatVersion {
    /// Return whether a reader supports this backward-compatible format.
    pub fn supported_by(self, reader: Self) -> bool {
        self.major == reader.major && self.minor <= reader.minor
    }
}
