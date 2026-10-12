//! Tracks the portal's lifecycle and decides when it should shut down.
//!
//! The deadline is:
//! - before the portal page has been opened: start + `--activity-timeout`
//!   (the original WiFi Connect behaviour);
//! - after it has been opened: last request from a client + `--idle-timeout`;
//! - while a submitted network is being connected: none.
//!
//! With `--status-file`, every change is written there as JSON so another
//! process can show exactly when the portal will close. Times are seconds on
//! the system uptime clock (the first field of `/proc/uptime`), which is not
//! affected by wall-clock changes.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process;
use std::sync::Mutex;
use std::time::Instant;

use serde_json;

use config::Config;

const STATUS_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PortalState {
    Waiting,
    Opened,
    Connecting,
    Exited,
}

#[derive(Serialize)]
struct StatusRecord {
    version: u32,
    pid: u32,
    state: PortalState,
    exit_reason: Option<&'static str>,
    started_at: f64,
    opened_at: Option<f64>,
    last_activity_at: Option<f64>,
    activity_timeout: u64,
    idle_timeout: u64,
    deadline: Option<f64>,
    updated_at: f64,
}

struct Inner {
    state: PortalState,
    exit_reason: Option<&'static str>,
    started_at: f64,
    opened_at: Option<f64>,
    last_activity_at: Option<f64>,
}

pub struct PortalStatus {
    path: Option<PathBuf>,
    activity_timeout: u64,
    idle_timeout: u64,
    base_uptime: f64,
    base_instant: Instant,
    inner: Mutex<Inner>,
}

impl PortalStatus {
    pub fn new(config: &Config) -> PortalStatus {
        let base_uptime = read_uptime().unwrap_or_else(|| {
            warn!("Cannot read /proc/uptime; status file times will be relative to start");
            0.0
        });

        let status = PortalStatus {
            path: config.status_file.clone(),
            activity_timeout: config.activity_timeout,
            idle_timeout: config.idle_timeout,
            base_uptime,
            base_instant: Instant::now(),
            inner: Mutex::new(Inner {
                state: PortalState::Waiting,
                exit_reason: None,
                started_at: base_uptime,
                opened_at: None,
                last_activity_at: None,
            }),
        };

        status.write(&status.inner.lock().unwrap());

        status
    }

    /// Whether any timeout is configured, i.e. whether the portal can ever
    /// close on its own.
    pub fn has_timeouts(&self) -> bool {
        self.activity_timeout > 0 || self.idle_timeout > 0
    }

    /// A request from a client reached the portal's web server.
    pub fn record_activity(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.last_activity_at = Some(self.now());
        self.write(&inner);
    }

    /// A client opened the portal page (requested the network list).
    pub fn mark_opened(&self) {
        let mut inner = self.inner.lock().unwrap();
        let now = self.now();
        if inner.opened_at.is_none() {
            inner.opened_at = Some(now);
        }
        inner.last_activity_at = Some(now);
        if inner.state == PortalState::Waiting {
            inner.state = PortalState::Opened;
        }
        self.write(&inner);
    }

    /// Connecting to a submitted network; no deadline while this lasts.
    pub fn mark_connecting(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.state = PortalState::Connecting;
        self.write(&inner);
    }

    /// The submitted network did not connect and the portal is back. The
    /// idle clock restarts so the user gets a full window to try again.
    pub fn mark_connect_failed(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.last_activity_at = Some(self.now());
        inner.state = if inner.opened_at.is_some() {
            PortalState::Opened
        } else {
            PortalState::Waiting
        };
        self.write(&inner);
    }

    pub fn mark_exited(&self, reason: &'static str) {
        let mut inner = self.inner.lock().unwrap();
        if inner.state == PortalState::Exited {
            return;
        }
        inner.state = PortalState::Exited;
        inner.exit_reason = Some(reason);
        self.write(&inner);
    }

    pub fn is_opened(&self) -> bool {
        self.inner.lock().unwrap().opened_at.is_some()
    }

    pub fn deadline_passed(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        match self.deadline(&inner) {
            Some(deadline) => self.now() >= deadline,
            None => false,
        }
    }

    fn deadline(&self, inner: &Inner) -> Option<f64> {
        match inner.state {
            PortalState::Connecting | PortalState::Exited => None,
            PortalState::Waiting | PortalState::Opened => {
                if inner.opened_at.is_some() {
                    if self.idle_timeout == 0 {
                        return None;
                    }
                    let last = inner.last_activity_at.unwrap_or(inner.started_at);
                    Some(last + self.idle_timeout as f64)
                } else if self.activity_timeout > 0 {
                    Some(inner.started_at + self.activity_timeout as f64)
                } else {
                    None
                }
            }
        }
    }

    fn now(&self) -> f64 {
        let elapsed = self.base_instant.elapsed();
        self.base_uptime + elapsed.as_secs() as f64 + f64::from(elapsed.subsec_nanos()) / 1e9
    }

    fn write(&self, inner: &Inner) {
        let path = match self.path {
            Some(ref path) => path,
            None => return,
        };

        let record = StatusRecord {
            version: STATUS_VERSION,
            pid: process::id(),
            state: inner.state,
            exit_reason: inner.exit_reason,
            started_at: inner.started_at,
            opened_at: inner.opened_at,
            last_activity_at: inner.last_activity_at,
            activity_timeout: self.activity_timeout,
            idle_timeout: self.idle_timeout,
            deadline: self.deadline(inner),
            updated_at: self.now(),
        };

        if let Err(e) = write_atomically(path, &record) {
            warn!("Writing status file {:?} failed: {}", path, e);
        }
    }
}

fn write_atomically(path: &PathBuf, record: &StatusRecord) -> ::std::io::Result<()> {
    let json = serde_json::to_string(record)
        .map_err(|e| ::std::io::Error::new(::std::io::ErrorKind::Other, e))?;

    let mut tmp = path.clone().into_os_string();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);

    {
        let mut file = fs::File::create(&tmp)?;
        file.write_all(json.as_bytes())?;
        file.write_all(b"\n")?;
    }

    fs::rename(&tmp, path)
}

fn read_uptime() -> Option<f64> {
    let contents = fs::read_to_string("/proc/uptime").ok()?;
    contents.split_whitespace().next()?.parse().ok()
}
