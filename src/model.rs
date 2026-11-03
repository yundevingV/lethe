#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheGroup {
    Disk,
    Docker,
    Packages,
}

#[derive(Clone, Debug)]
pub struct CleanResult {
    pub ok: bool,
    pub message: String,
}

impl CleanResult {
    pub fn ok(message: impl Into<String>) -> Self {
        Self { ok: true, message: message.into() }
    }

    pub fn fail(message: impl Into<String>) -> Self {
        Self { ok: false, message: message.into() }
    }
}

#[derive(Clone, Debug)]
pub struct CacheRow {
    pub id: String,
    pub title: String,
    pub detail: String,
    pub group: CacheGroup,
    pub bytes: Option<u64>,
    pub blocked_reason: Option<String>,
    pub selected: bool,
    pub status: Option<String>,
    pub status_is_error: bool,
    pub cleanable: bool,
}

impl CacheRow {
    pub fn can_select(&self, busy: bool) -> bool {
        if busy || !self.cleanable || self.blocked_reason.is_some() {
            return false;
        }
        self.bytes.unwrap_or(0) > 0
    }
}
