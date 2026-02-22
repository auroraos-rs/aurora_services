use crate::backends::{Bus, DBusBackend, MessageItem};
use crate::error::{AuroraError, Result};
use dbus::arg::messageitem::MessageItemArray;
use std::collections::HashMap;

const NOTIFICATIONS_SERVICE: &str = "org.freedesktop.Notifications";
const NOTIFICATIONS_PATH: &str = "/org/freedesktop/Notifications";
const NOTIFICATIONS_INTERFACE: &str = "org.freedesktop.Notifications";

#[derive(Debug, Clone)]
pub struct Notification {
    pub app_name: String,
    pub replaces_id: u32,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub actions: Vec<String>,
    pub hints: HashMap<String, VariantValue>,
    pub expire_timeout: i32,
}

impl Default for Notification {
    fn default() -> Self {
        Self {
            app_name: String::new(),
            replaces_id: 0,
            app_icon: String::new(),
            summary: String::new(),
            body: String::new(),
            actions: Vec::new(),
            hints: HashMap::new(),
            expire_timeout: -1,
        }
    }
}

#[derive(Debug, Clone)]
pub enum VariantValue {
    Int(i32),
    UInt(u32),
    Double(f64),
    String(String),
    Bool(bool),
    Byte(u8),
    ByteArray(Vec<u8>),
}

impl VariantValue {
    fn to_message_item(&self) -> MessageItem {
        match self {
            VariantValue::Int(i) => MessageItem::Int32(*i),
            VariantValue::UInt(u) => MessageItem::UInt32(*u),
            VariantValue::Double(d) => MessageItem::Double(*d),
            VariantValue::String(s) => MessageItem::Str(s.clone()),
            VariantValue::Bool(b) => MessageItem::Bool(*b),
            VariantValue::Byte(b) => MessageItem::Byte(*b),
            VariantValue::ByteArray(data) => {
                MessageItem::new_array(data.iter().map(|b| MessageItem::Byte(*b)).collect())
                    .unwrap_or_else(|_| MessageItem::Str("[]".to_string()))
            }
        }
    }
}

pub struct NotificationService {
    dbus: DBusBackend,
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new().expect("Failed to create NotificationService")
    }
}

impl NotificationService {
    pub fn new() -> Result<Self> {
        Ok(Self::with_backend(DBusBackend::new()?))
    }

    pub fn with_backend(dbus: DBusBackend) -> Self {
        Self { dbus }
    }

    pub fn notify(&self, notification: &Notification) -> Result<u32> {
        let actions_array = pack_actions(notification);
        let hints_dict = pack_hints(notification)?;

        let args = vec![
            MessageItem::Str(notification.app_name.clone()),
            MessageItem::UInt32(notification.replaces_id),
            MessageItem::Str(notification.app_icon.clone()),
            MessageItem::Str(notification.summary.clone()),
            MessageItem::Str(notification.body.clone()),
            actions_array,
            hints_dict,
            MessageItem::Int32(notification.expire_timeout),
        ];

        let result = self.dbus.call_method(
            Bus::Session,
            NOTIFICATIONS_SERVICE,
            NOTIFICATIONS_PATH,
            NOTIFICATIONS_INTERFACE,
            "Notify",
            &args,
        )?;

        result
            .first()
            .and_then(|item| {
                if let MessageItem::UInt32(id) = item {
                    Some(*id)
                } else {
                    None
                }
            })
            .ok_or_else(|| AuroraError::DBus("Invalid response from Notify".to_string()))
    }

    pub fn close(&self, id: u32) -> Result<()> {
        self.dbus.call_method(
            Bus::Session,
            NOTIFICATIONS_SERVICE,
            NOTIFICATIONS_PATH,
            NOTIFICATIONS_INTERFACE,
            "CloseNotification",
            &[MessageItem::UInt32(id)],
        )?;

        Ok(())
    }

    pub fn get_capabilities(&self) -> Result<Vec<String>> {
        let result = self.dbus.call_method(
            Bus::Session,
            NOTIFICATIONS_SERVICE,
            NOTIFICATIONS_PATH,
            NOTIFICATIONS_INTERFACE,
            "GetCapabilities",
            &[],
        )?;

        let capabilities = result
            .first()
            .and_then(|item| {
                if let MessageItem::Array(arr) = item {
                    Some(
                        arr.iter()
                            .filter_map(|i| {
                                if let MessageItem::Str(s) = i {
                                    Some(s.clone())
                                } else {
                                    None
                                }
                            })
                            .collect(),
                    )
                } else {
                    None
                }
            })
            .unwrap_or_default();

        Ok(capabilities)
    }

    pub fn get_server_info(&self) -> Result<ServerInfo> {
        let result = self.dbus.call_method(
            Bus::Session,
            NOTIFICATIONS_SERVICE,
            NOTIFICATIONS_PATH,
            NOTIFICATIONS_INTERFACE,
            "GetServerInformation",
            &[],
        )?;

        let mut iter = result.into_iter();

        let name = iter
            .next()
            .and_then(|item| {
                if let MessageItem::Str(s) = item {
                    Some(s)
                } else {
                    None
                }
            })
            .unwrap_or_default();

        let vendor = iter
            .next()
            .and_then(|item| {
                if let MessageItem::Str(s) = item {
                    Some(s)
                } else {
                    None
                }
            })
            .unwrap_or_default();

        let version = iter
            .next()
            .and_then(|item| {
                if let MessageItem::Str(s) = item {
                    Some(s)
                } else {
                    None
                }
            })
            .unwrap_or_default();

        let spec_version = iter
            .next()
            .and_then(|item| {
                if let MessageItem::Str(s) = item {
                    Some(s)
                } else {
                    None
                }
            })
            .unwrap_or_default();

        Ok(ServerInfo {
            name,
            vendor,
            version,
            spec_version,
        })
    }
}

fn pack_actions(notification: &Notification) -> MessageItem {
    if !notification.actions.is_empty() {
        let actions: Vec<MessageItem> = notification
            .actions
            .iter()
            .map(|a| MessageItem::Str(a.clone()))
            .collect();
        if let Ok(array) = MessageItem::new_array(actions) {
            return array;
        }
    }
    MessageItem::Array(MessageItemArray::new(vec![], "as".into()).unwrap())
}

fn pack_hints(notification: &Notification) -> Result<MessageItem> {
    if !notification.hints.is_empty() {
        let hints: Vec<(MessageItem, MessageItem)> = notification
            .hints
            .iter()
            .map(|(k, v)| {
                (
                    MessageItem::Str(k.clone()),
                    MessageItem::Variant(Box::new(v.to_message_item())),
                )
            })
            .collect();

        if let Ok(array) = MessageItem::new_dict(hints) {
            return Ok(array);
        }
    }
    Ok(MessageItem::Array(
        MessageItemArray::new(vec![], "a{sv}".into()).unwrap(),
    ))
}

#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub name: String,
    pub vendor: String,
    pub version: String,
    pub spec_version: String,
}

pub struct NotificationBuilder {
    notification: Notification,
}

impl NotificationBuilder {
    pub fn new() -> Self {
        Self {
            notification: Notification::default(),
        }
    }

    pub fn app_name(mut self, name: &str) -> Self {
        self.notification.app_name = name.to_string();
        self
    }

    pub fn summary(mut self, summary: &str) -> Self {
        self.notification.summary = summary.to_string();
        self
    }

    pub fn body(mut self, body: &str) -> Self {
        self.notification.body = body.to_string();
        self
    }

    pub fn icon(mut self, icon: &str) -> Self {
        self.notification.app_icon = icon.to_string();
        self
    }

    pub fn timeout(mut self, timeout_ms: i32) -> Self {
        self.notification.expire_timeout = timeout_ms;
        self
    }

    pub fn replaces(mut self, id: u32) -> Self {
        self.notification.replaces_id = id;
        self
    }

    pub fn action(mut self, action: &str) -> Self {
        self.notification.actions.push(action.to_string());
        self
    }

    pub fn hint(mut self, key: &str, value: VariantValue) -> Self {
        self.notification.hints.insert(key.to_string(), value);
        self
    }

    pub fn urgency(mut self, urgency: u8) -> Self {
        self.notification
            .hints
            .insert("urgency".to_string(), VariantValue::Byte(urgency));
        self
    }

    pub fn build(self) -> Notification {
        self.notification
    }
}

impl Default for NotificationBuilder {
    fn default() -> Self {
        Self::new()
    }
}
