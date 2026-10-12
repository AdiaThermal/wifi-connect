//! Watches the WiFi device's NetworkManager state changes during a connection
//! attempt, so an extended activation wait (`--activation-timeout`) can end as
//! soon as NetworkManager rejects the credentials instead of running out the
//! clock on a wrong password.
//!
//! Only a disconnect during the WPA handshake counts as rejected credentials.
//! NetworkManager also enters NEED_AUTH briefly at the start of every
//! activation (reason NONE) and when association times out (reason
//! SUPPLICANT_TIMEOUT, e.g. a router ignoring the client); neither means the
//! password is wrong.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use dbus::{BusType, Connection, ConnectionItem, Message, Path};

const NM_SERVICE: &str = "org.freedesktop.NetworkManager";
const NM_PATH: &str = "/org/freedesktop/NetworkManager";
const NM_DEVICE_INTERFACE: &str = "org.freedesktop.NetworkManager.Device";

// NMDeviceState / NMDeviceStateReason values (libnm nm-dbus-interface.h).
const NM_DEVICE_STATE_NEED_AUTH: u32 = 60;
const NM_DEVICE_STATE_REASON_SUPPLICANT_DISCONNECT: u32 = 8;

const POLL_MS: i32 = 200;
const DBUS_TIMEOUT_MS: i32 = 5000;

pub struct AuthWatch {
    stop: Arc<AtomicBool>,
    rejected: Arc<AtomicBool>,
}

impl AuthWatch {
    /// Start watching `interface`. Returns `None` (and the caller simply waits
    /// without early exit) if the watch could not be set up.
    pub fn start(interface: &str) -> Option<AuthWatch> {
        let stop = Arc::new(AtomicBool::new(false));
        let rejected = Arc::new(AtomicBool::new(false));
        let (ready_tx, ready_rx) = channel();

        let interface = interface.to_string();
        let thread_stop = stop.clone();
        let thread_rejected = rejected.clone();

        // A dbus::Connection is not Send, so it is created on the watcher
        // thread itself.
        thread::spawn(move || {
            let connection = match subscribe(&interface) {
                Ok(connection) => {
                    let _ = ready_tx.send(true);
                    connection
                }
                Err(e) => {
                    warn!("Cannot watch {} for rejected credentials: {}", interface, e);
                    let _ = ready_tx.send(false);
                    return;
                }
            };

            for item in connection.iter(POLL_MS) {
                if thread_stop.load(Ordering::SeqCst) {
                    break;
                }

                if let ConnectionItem::Signal(message) = item {
                    let (new_state, _, reason): (Option<u32>, Option<u32>, Option<u32>) =
                        message.get3();

                    if new_state == Some(NM_DEVICE_STATE_NEED_AUTH)
                        && reason == Some(NM_DEVICE_STATE_REASON_SUPPLICANT_DISCONNECT)
                    {
                        thread_rejected.store(true, Ordering::SeqCst);
                    }
                }
            }
        });

        match ready_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(true) => Some(AuthWatch { stop, rejected }),
            _ => {
                stop.store(true, Ordering::SeqCst);
                None
            }
        }
    }

    /// NetworkManager asked for new secrets after the WPA handshake failed
    /// since the watch started.
    pub fn credentials_rejected(&self) -> bool {
        self.rejected.load(Ordering::SeqCst)
    }
}

impl Drop for AuthWatch {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

fn subscribe(interface: &str) -> Result<Connection, String> {
    let connection = Connection::get_private(BusType::System).map_err(|e| format!("{:?}", e))?;

    let message = Message::new_method_call(NM_SERVICE, NM_PATH, NM_SERVICE, "GetDeviceByIpIface")?
        .append1(interface);
    let reply = connection
        .send_with_reply_and_block(message, DBUS_TIMEOUT_MS)
        .map_err(|e| format!("{:?}", e))?;
    let device_path: Path = reply
        .get1()
        .ok_or_else(|| "unexpected GetDeviceByIpIface reply".to_string())?;
    let device_path = device_path.to_string();

    let rule = format!(
        "type='signal',sender='{}',interface='{}',member='StateChanged',path='{}'",
        NM_SERVICE, NM_DEVICE_INTERFACE, device_path
    );
    connection.add_match(&rule).map_err(|e| format!("{:?}", e))?;

    Ok(connection)
}
