use crate::error::{AuroraError, Result};
use dbus::arg;
pub use dbus::arg::messageitem::MessageItem;
use dbus::blocking::{BlockingSender, Connection};
use dbus::message::Message;
use std::sync::Mutex;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bus {
    System,
    Session,
}

pub struct DBusBackend {
    system_conn: Mutex<Connection>,
    session_conn: Mutex<Connection>,
}

impl Default for DBusBackend {
    fn default() -> Self {
        Self::new().expect("Failed to connect to DBus")
    }
}

impl DBusBackend {
    pub fn new() -> Result<Self> {
        let system_conn =
            Connection::new_system().map_err(|e| AuroraError::ConnectionFailed(e.to_string()))?;

        let session_conn =
            Connection::new_session().map_err(|e| AuroraError::ConnectionFailed(e.to_string()))?;

        Ok(Self {
            system_conn: Mutex::new(system_conn),
            session_conn: Mutex::new(session_conn),
        })
    }

    pub fn with_connections(system_conn: Connection, session_conn: Connection) -> Self {
        Self {
            system_conn: Mutex::new(system_conn),
            session_conn: Mutex::new(session_conn),
        }
    }

    fn get_connection(&self, bus: Bus) -> Result<std::sync::MutexGuard<'_, Connection>> {
        let conn = match bus {
            Bus::System => &self.system_conn,
            Bus::Session => &self.session_conn,
        };
        conn.lock().map_err(|e| AuroraError::DBus(e.to_string()))
    }

    pub fn call_method(
        &self,
        bus: Bus,
        destination: &str,
        path: &str,
        interface: &str,
        method: &str,
        args: &[MessageItem],
    ) -> Result<Vec<MessageItem>> {
        let conn = self.get_connection(bus)?;

        let msg = Message::new_method_call(destination, path, interface, method)
            .map_err(|e| AuroraError::DBus(e.to_string()))?;

        let msg = args.iter().cloned().fold(msg, Message::append1);

        let reply = conn
            .send_with_reply_and_block(msg, Duration::from_millis(5000))
            .map_err(|e| AuroraError::DBus(e.to_string()))?;

        Ok(reply.get_items())
    }

    pub fn get_property(
        &self,
        bus: Bus,
        destination: &str,
        path: &str,
        interface: &str,
        property: &str,
    ) -> Result<MessageItem> {
        let conn = self.get_connection(bus)?;

        let msg =
            Message::new_method_call(destination, path, "org.freedesktop.DBus.Properties", "Get")
                .map_err(|e| AuroraError::DBus(e.to_string()))?
                .append2(interface, property);

        let reply = conn
            .send_with_reply_and_block(msg, Duration::from_millis(5000))
            .map_err(|e| AuroraError::DBus(e.to_string()))?;

        let items = reply.get_items();
        if items.is_empty() {
            return Err(AuroraError::PropertyNotFound(property.to_string()));
        }

        Ok(items.into_iter().next().unwrap())
    }

    pub fn set_property(
        &self,
        bus: Bus,
        destination: &str,
        path: &str,
        interface: &str,
        property: &str,
        value: MessageItem,
    ) -> Result<()> {
        let conn = self.get_connection(bus)?;

        let msg =
            Message::new_method_call(destination, path, "org.freedesktop.DBus.Properties", "Set")
                .map_err(|e| AuroraError::DBus(e.to_string()))?
                .append3(interface, property, value);

        conn.send_with_reply_and_block(msg, Duration::from_millis(5000))
            .map_err(|e| AuroraError::DBus(e.to_string()))?;

        Ok(())
    }

    pub fn call_method_simple<R: for<'b> arg::Get<'b>>(
        &self,
        bus: Bus,
        destination: &str,
        path: &str,
        interface: &str,
        method: &str,
    ) -> Result<R> {
        let conn = self.get_connection(bus)?;

        let msg = Message::new_method_call(destination, path, interface, method)
            .map_err(|e| AuroraError::DBus(e.to_string()))?;

        let reply = conn
            .send_with_reply_and_block(msg, Duration::from_millis(5000))
            .map_err(|e| AuroraError::DBus(e.to_string()))?;

        reply
            .get1()
            .ok_or_else(|| AuroraError::DBus("No return value".to_string()))
    }
}
