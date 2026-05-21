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

    fn reconnect(&self, bus: Bus) -> Result<()> {
        match bus {
            Bus::System => {
                let new_conn = Connection::new_system()
                    .map_err(|e| AuroraError::ConnectionFailed(e.to_string()))?;
                let mut guard = self
                    .system_conn
                    .lock()
                    .map_err(|e| AuroraError::DBus(e.to_string()))?;
                *guard = new_conn;
            }
            Bus::Session => {
                let new_conn = Connection::new_session()
                    .map_err(|e| AuroraError::ConnectionFailed(e.to_string()))?;
                let mut guard = self
                    .session_conn
                    .lock()
                    .map_err(|e| AuroraError::DBus(e.to_string()))?;
                *guard = new_conn;
            }
        }
        Ok(())
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
        self.call_method_internal(bus, destination, path, interface, method, args, true)
    }

    fn call_method_internal(
        &self,
        bus: Bus,
        destination: &str,
        path: &str,
        interface: &str,
        method: &str,
        args: &[MessageItem],
        allow_reconnect: bool,
    ) -> Result<Vec<MessageItem>> {
        let result = {
            let conn = self.get_connection(bus)?;
            let msg = Message::new_method_call(destination, path, interface, method)
                .map_err(|e| AuroraError::DBus(e.to_string()))?;
            let msg = args.iter().cloned().fold(msg, Message::append1);
            conn.send_with_reply_and_block(msg, Duration::from_millis(5000))
        };

        match result {
            Ok(reply) => Ok(reply.get_items()),
            Err(e) => {
                if allow_reconnect && is_connection_error(&e) {
                    self.reconnect(bus)?;
                    self.call_method_internal(bus, destination, path, interface, method, args, false)
                } else {
                    Err(AuroraError::DBus(e.to_string()))
                }
            }
        }
    }

    pub fn get_property(
        &self,
        bus: Bus,
        destination: &str,
        path: &str,
        interface: &str,
        property: &str,
    ) -> Result<MessageItem> {
        self.get_property_internal(bus, destination, path, interface, property, true)
    }

    fn get_property_internal(
        &self,
        bus: Bus,
        destination: &str,
        path: &str,
        interface: &str,
        property: &str,
        allow_reconnect: bool,
    ) -> Result<MessageItem> {
        let result = {
            let conn = self.get_connection(bus)?;
            let msg =
                Message::new_method_call(destination, path, "org.freedesktop.DBus.Properties", "Get")
                    .map_err(|e| AuroraError::DBus(e.to_string()))?
                    .append2(interface, property);
            conn.send_with_reply_and_block(msg, Duration::from_millis(5000))
        };

        match result {
            Ok(reply) => {
                let items = reply.get_items();
                if items.is_empty() {
                    return Err(AuroraError::PropertyNotFound(property.to_string()));
                }
                Ok(items.into_iter().next().unwrap())
            }
            Err(e) => {
                if allow_reconnect && is_connection_error(&e) {
                    self.reconnect(bus)?;
                    self.get_property_internal(bus, destination, path, interface, property, false)
                } else {
                    Err(AuroraError::DBus(e.to_string()))
                }
            }
        }
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
        self.set_property_internal(bus, destination, path, interface, property, value, true)
    }

    fn set_property_internal(
        &self,
        bus: Bus,
        destination: &str,
        path: &str,
        interface: &str,
        property: &str,
        value: MessageItem,
        allow_reconnect: bool,
    ) -> Result<()> {
        let result = {
            let conn = self.get_connection(bus)?;
            let msg =
                Message::new_method_call(destination, path, "org.freedesktop.DBus.Properties", "Set")
                    .map_err(|e| AuroraError::DBus(e.to_string()))?
                    .append3(interface, property, value.clone());
            conn.send_with_reply_and_block(msg, Duration::from_millis(5000))
        };

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                if allow_reconnect && is_connection_error(&e) {
                    self.reconnect(bus)?;
                    self.set_property_internal(
                        bus, destination, path, interface, property, value, false,
                    )
                } else {
                    Err(AuroraError::DBus(e.to_string()))
                }
            }
        }
    }

    pub fn call_method_simple<R: for<'b> arg::Get<'b>>(
        &self,
        bus: Bus,
        destination: &str,
        path: &str,
        interface: &str,
        method: &str,
    ) -> Result<R> {
        self.call_method_simple_internal(bus, destination, path, interface, method, true)
    }

    fn call_method_simple_internal<R: for<'b> arg::Get<'b>>(
        &self,
        bus: Bus,
        destination: &str,
        path: &str,
        interface: &str,
        method: &str,
        allow_reconnect: bool,
    ) -> Result<R> {
        let result = {
            let conn = self.get_connection(bus)?;
            let msg = Message::new_method_call(destination, path, interface, method)
                .map_err(|e| AuroraError::DBus(e.to_string()))?;
            conn.send_with_reply_and_block(msg, Duration::from_millis(5000))
        };

        match result {
            Ok(reply) => reply
                .get1()
                .ok_or_else(|| AuroraError::DBus("No return value".to_string())),
            Err(e) => {
                if allow_reconnect && is_connection_error(&e) {
                    self.reconnect(bus)?;
                    self.call_method_simple_internal(bus, destination, path, interface, method, false)
                } else {
                    Err(AuroraError::DBus(e.to_string()))
                }
            }
        }
    }
}

fn is_connection_error(err: &dbus::Error) -> bool {
    if let Some(name) = err.name() {
        if name == "org.freedesktop.DBus.Error.Disconnected"
            || name == "org.freedesktop.DBus.Error.NoServer"
        {
            return true;
        }
    }
    let msg = err.message().unwrap_or("").to_lowercase();
    msg.contains("disconnected")
        || msg.contains("broken pipe")
        || msg.contains("connection reset")
}
