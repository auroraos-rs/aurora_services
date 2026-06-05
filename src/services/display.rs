use crate::backends::{Bus, DBusBackend};
use crate::error::Result;

const MCE_SERVICE: &str = "com.nokia.mce";
const MCE_PATH: &str = "/com/nokia/mce/request";
const MCE_INTERFACE: &str = "com.nokia.mce.request";

pub struct DisplayService {
    dbus: DBusBackend,
}

impl DisplayService {
    pub fn new() -> Result<Self> {
        Ok(Self {
            dbus: DBusBackend::new()?,
        })
    }

    /// Requests a pause in display blanking.
    /// Pause activates into 60 sec.
    pub fn pause_display_blanking(&self) -> Result<()> {
        self.dbus
            .call_method(
                Bus::System,
                MCE_SERVICE,
                MCE_PATH,
                MCE_INTERFACE,
                "req_display_blanking_pause",
                &[],
            )
            .map(|_| ())
    }
}
