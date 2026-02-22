# Спецификация библиотеки aurora_services

## Обзор

Библиотека для работы с системными конфигурациями и сервисами Aurora OS через DBus и DConf.

---

## Архитектура

```
src/
├── lib.rs              # Публичный API, реэкспорт
├── error.rs            # Типы ошибок
├── backends/
│   ├── mod.rs
│   ├── dbus.rs         # DBus backend
│   └── dconf.rs        # DConf backend
└── services/
    ├── mod.rs
    ├── notifications.rs
    └── settings.rs
```

---

## Модуль: error.rs

### Назначение
Централизованная обработка ошибок библиотеки.

### Реализация

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuroraError {
    #[error("DBus error: {0}")]
    DBus(String),
    
    #[error("DConf error: {0}")]
    DConf(String),
    
    #[error("Property not found: {0}")]
    PropertyNotFound(String),
    
    #[error("Invalid value type: expected {expected}, got {actual}")]
    InvalidType { expected: String, actual: String },
    
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
}
```

### Тип возвращаемого значения
Все публичные методы используют `Result<T, AuroraError>`.

---

## Модуль: backends/dbus.rs

### Назначение
Низкоуровневый клиент для работы с DBus (системная и сессионная шины).

### Структуры

```rust
use dbus::{Connection, BusType};

pub struct DBusBackend {
    system_conn: Connection,
    session_conn: Connection,
}

impl Default for DBusBackend {
    fn default() -> Self {
        Self::new().expect("Failed to connect to DBus")
    }
}
```

### Конструкторы

| Метод | Описание |
|-------|----------|
| `new() -> Result<Self, AuroraError>` | Подключается к обеим шинам |
| `with_connections(system, session) -> Self` | Для инъекции извне |

### Методы

```rust
impl DBusBackend {
    /// Вызвать метод на DBus
    pub fn call_method(
        &self,
        bus: BusType,
        destination: &str,
        path: &str,
        interface: &str,
        method: &str,
        args: &[MessageItem],
    ) -> Result<Vec<MessageItem>, AuroraError>;
    
    /// Получить свойство
    pub fn get_property(
        &self,
        bus: BusType,
        destination: &str,
        path: &str,
        interface: &str,
        property: &str,
    ) -> Result<MessageItem, AuroraError>;
    
    /// Установить свойство
    pub fn set_property(
        &self,
        bus: BusType,
        destination: &str,
        path: &str,
        interface: &str,
        property: &str,
        value: MessageItem,
    ) -> Result<(), AuroraError>;
}
```

### Типы шин

```rust
pub enum Bus {
    System,
    Session,
}
```

### Внутренняя реализация

**Подключение:**
```rust
fn new() -> Result<Self, AuroraError> {
    let system_conn = Connection::get_private(BusType::System)
        .map_err(|e| AuroraError::ConnectionFailed(e.to_string()))?;
    
    let session_conn = Connection::get_private(BusType::Session)
        .map_err(|e| AuroraError::ConnectionFailed(e.to_string()))?;
    
    Ok(Self { system_conn, session_conn })
}
```

**Вызов метода:**
```rust
fn call_method(...) -> Result<Vec<MessageItem>, AuroraError> {
    let conn = match bus {
        BusType::System => &self.system_conn,
        BusType::Session => &self.session_conn,
    };
    
    let msg = Message::new_method_call(destination, path, interface, method)
        .map_err(|e| AuroraError::DBus(e.to_string()))?
        .append_all(args);
    
    let reply = conn.send_with_reply_and_block(msg, 5000)
        .map_err(|e| AuroraError::DBus(e.to_string()))?;
    
    Ok(reply.get_items())
}
```

---

## Модуль: backends/dconf.rs

### Назначение
Клиент для чтения и записи настроек DConf.

### Как работает DConf в Aurora OS

DConf хранит настройки в виде ключей по путям:
- Путь: `/desktop/jolla/theme/color`
- Ключ: `highlight`
- Значение: `'#ff90a3fe'`

DConf использует DBus-сервис `ca.desrt.dconf` для синхронизации изменений между процессами.

### Структуры

```rust
pub struct DConfBackend {
    // Без полей — использует CLI команды
}
```

### Конструкторы

```rust
impl DConfBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for DConfBackend {
    fn default() -> Self {
        Self::new()
    }
}
```

### Методы

```rust
impl DConfBackend {
    /// Прочитать значение ключа
    pub fn get(&self, path: &str, key: &str) -> Result<DConfValue, AuroraError>;
    
    /// Записать значение ключа
    pub fn set(&self, path: &str, key: &str, value: DConfValue) -> Result<(), AuroraError>;
    
    /// Прочитать все ключи по пути
    pub fn get_all(&self, path: &str) -> Result<Vec<(String, DConfValue)>, AuroraError>;
}
```

### Тип значений DConf

```rust
#[derive(Debug, Clone)]
pub enum DConfValue {
    String(String),
    Int(i32),
    Float(f64),
    Bool(bool),
    Array(Vec<DConfValue>),
    Dict(HashMap<String, DConfValue>),
}
```

### Внутренняя реализация

**Чтение значения:**

DConf не предоставляет прямой DBus API для чтения. Чтение происходит через файл базы данных:
```
~/.config/dconf/user
```

Варианты реализации:
1. **GSettings CLI** — вызов `gsettings get schema key` (если доступен)
2. **dconf CLI** — вызов `dconf read /path/key`
3. **Прямое чтение базы** — бинарный формат GVariant

**Рекомендация:** использовать `dconf read` / `dconf write` команды:

```rust
fn get(&self, path: &str, key: &str) -> Result<DConfValue, AuroraError> {
    let full_path = format!("{}/{}", path, key);
    let output = Command::new("dconf")
        .arg("read")
        .arg(&full_path)
        .output()
        .map_err(|e| AuroraError::DConf(e.to_string()))?;
    
    let value_str = String::from_utf8_lossy(&output.stdout).trim();
    parse_dconf_value(value_str)
}
```

**Парсинг значений:**

```
'string'     → DConfValue::String("string")
42           → DConfValue::Int(42)
3.14         → DConfValue::Float(3.14)
true         → DConfValue::Bool(true)
['a', 'b']   → DConfValue::Array([...)
```

**Запись значения:**

```rust
fn set(&self, path: &str, key: &str, value: DConfValue) -> Result<(), AuroraError> {
    let full_path = format!("{}/{}", path, key);
    let value_str = value_to_dconf_string(value);
    
    Command::new("dconf")
        .arg("write")
        .arg(&full_path)
        .arg(&value_str)
        .status()
        .map_err(|e| AuroraError::DConf(e.to_string()))?;
    
    Ok(())
}
```

### Таблица путей DConf

| Путь | Ключи | Описание |
|------|-------|----------|
| `/desktop/jolla/theme` | `active_ambience`, `color_scheme` | Тема оформления |
| `/desktop/jolla/theme/color` | `highlight`, `primary`, `secondary` | Цвета темы |
| `/desktop/jolla/background/portrait` | `home_picture_filename` | Обои |
| `/jolla/sound` | `theme` | Звуковая тема |
| `/lipstick` | `orientationLock` | Блокировка ориентации |
| `/lipstick/screen/primary` | `height`, `width` | Размер экрана |

---

## Модуль: services/notifications.rs

### Назначение
Отправка системных уведомлений через DBus.

### DBus интерфейс

```
Service: org.freedesktop.Notifications
Path:    /org/freedesktop/Notifications
Interface: org.freedesktop.Notifications
```

### Структуры

```rust
pub struct Notification {
    pub app_name: String,
    pub replaces_id: u32,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    pub actions: Vec<String>,
    pub hints: HashMap<String, Variant>,
    pub expire_timeout: i32,
}

pub struct NotificationService {
    dbus: DBusBackend,
}
```

### Конструкторы

```rust
impl NotificationService {
    pub fn new() -> Result<Self, AuroraError> {
        Ok(Self::with_backend(DBusBackend::new()?))
    }
    
    pub fn with_backend(dbus: DBusBackend) -> Self {
        Self { dbus }
    }
}
```

### Методы

```rust
impl NotificationService {
    /// Отправить уведомление, возвращает ID
    pub fn notify(&self, notification: &Notification) -> Result<u32, AuroraError>;
    
    /// Закрыть уведомление по ID
    pub fn close(&self, id: u32) -> Result<(), AuroraError>;
    
    /// Получить возможности сервера
    pub fn get_capabilities(&self) -> Result<Vec<String>, AuroraError>;
}
```

### Внутренняя реализация

```rust
fn notify(&self, notification: &Notification) -> Result<u32, AuroraError> {
    let args = [
        MessageItem::Str(notification.app_name.clone()),
        MessageItem::UInt32(notification.replaces_id),
        MessageItem::Str(notification.app_icon.clone()),
        MessageItem::Str(notification.summary.clone()),
        MessageItem::Str(notification.body.clone()),
        MessageItem::Array(
            notification.actions.iter()
                .map(|a| MessageItem::Str(a.clone()))
                .collect(),
            "s".into()
        ),
        MessageItem::Dict(...), // hints
        MessageItem::Int32(notification.expire_timeout),
    ];
    
    let result = self.dbus.call_method(
        BusType::Session,
        "org.freedesktop.Notifications",
        "/org/freedesktop/Notifications",
        "org.freedesktop.Notifications",
        "Notify",
        &args,
    )?;
    
    Ok(result[0].as_u32().unwrap_or(0))
}
```

---

## Модуль: services/settings.rs

### Назначение
Управление системными настройками (тема, звуки, экран и т.д.).

### Структуры

```rust
pub struct SettingsService {
    dconf: DConfBackend,
    dbus: DBusBackend,
}

pub struct ThemeSettings {
    pub active_ambience: String,
    pub color_scheme: u32,
    pub highlight_color: String,
    pub primary_color: String,
    pub secondary_color: String,
}

pub struct DisplaySettings {
    pub orientation_lock: Orientation,
    pub brightness: Option<u32>,
}

pub enum Orientation {
    Portrait,
    Landscape,
    Dynamic,
}
```

### Конструкторы

```rust
impl SettingsService {
    pub fn new() -> Result<Self, AuroraError> {
        Ok(Self::with_backend(DBusBackend::new()?))
    }
    
    pub fn with_backend(dbus: DBusBackend) -> Self {
        Self { 
            dconf: DConfBackend::new(),
            dbus,
        }
    }
}
```

### Методы

```rust
impl SettingsService {
    // Тема
    pub fn get_theme(&self) -> Result<ThemeSettings, AuroraError>;
    pub fn set_theme(&self, theme: &ThemeSettings) -> Result<(), AuroraError>;
    pub fn set_wallpaper(&self, path: &str) -> Result<(), AuroraError>;
    
    // Дисплей
    pub fn get_display(&self) -> Result<DisplaySettings, AuroraError>;
    pub fn set_orientation_lock(&self, orientation: Orientation) -> Result<(), AuroraError>;
    
    // Звук (через DBus profiled)
    pub fn get_sound_profile(&self) -> Result<String, AuroraError>;
    pub fn set_sound_profile(&self, profile: &str) -> Result<(), AuroraError>;
}
    pub fn get_theme(&self) -> Result<ThemeSettings, AuroraError>;
    pub fn set_theme(&self, theme: &ThemeSettings) -> Result<(), AuroraError>;
    pub fn set_wallpaper(&self, path: &str) -> Result<(), AuroraError>;
    
    // Дисплей
    pub fn get_display(&self) -> Result<DisplaySettings, AuroraError>;
    pub fn set_orientation_lock(&self, orientation: Orientation) -> Result<(), AuroraError>;
    
    // Звук (через DBus profiled)
    pub fn get_sound_profile(&self) -> Result<String, AuroraError>;
    pub fn set_sound_profile(&self, profile: &str) -> Result<(), AuroraError>;
}
```

### Внутренняя реализация

**Получение настроек темы:**

```rust
fn get_theme(&self) -> Result<ThemeSettings, AuroraError> {
    let active_ambience = self.dconf
        .get("/desktop/jolla/theme", "active_ambience")?
        .as_string()?;
    
    let color_scheme = self.dconf
        .get("/desktop/jolla/theme", "color_scheme")?
        .as_int()? as u32;
    
    let highlight = self.dconf
        .get("/desktop/jolla/theme/color", "highlight")?
        .as_string()?;
    
    // ... остальные цвета
    
    Ok(ThemeSettings { ... })
}
```

**Профили звука через DBus:**

Сервис `com.nokia.profiled` управляет профилями звука:
- `general` — обычный режим
- `silent` — без звука
- `meeting` — только вибрация

```rust
fn get_sound_profile(&self) -> Result<String, AuroraError> {
    // Через profiled или DConf
    // Путь в DConf: /sailfish/sound
}
```

---

## Модуль: lib.rs

### Реэкспорт

```rust
mod error;
mod backends;
mod services;

pub use error::AuroraError;
pub use backends::{DBusBackend, DConfBackend, DConfValue};
pub use services::{NotificationService, Notification, SettingsService, ThemeSettings, DisplaySettings};

// Удобные конструкторы
pub fn notifications() -> Result<NotificationService, AuroraError> {
    NotificationService::new()
}

pub fn settings() -> Result<SettingsService, AuroraError> {
    SettingsService::new()
}
```

---

## Примеры использования

### Отправка уведомления

```rust
use aurora_services::{NotificationService, Notification};

let service = NotificationService::new()?;

let id = service.notify(&Notification {
    app_name: "MyApp".into(),
    replaces_id: 0,
    app_icon: "icon-m-notifications".into(),
    summary: "Заголовок".into(),
    body: "Текст уведомления".into(),
    actions: vec![],
    hints: HashMap::new(),
    expire_timeout: 5000,
})?;
```

### Изменение темы

```rust
use aurora_services::{SettingsService, ThemeSettings};

let settings = SettingsService::new()?;

let theme = settings.get_theme()?;
println!("Current theme: {:?}", theme);

settings.set_wallpaper("/home/defaultuser/Pictures/wallpaper.jpg")?;
```

---

## Дополнительные сервисы (будущие)

| Сервис | Backend | Описание |
|--------|---------|----------|
| `BluetoothService` | DBus (org.bluez) | Управление Bluetooth |
| `UsbService` | DBus (com.meego.usb_moded) | Режимы USB |
| `SensorService` | DBus (com.nokia.SensorService) | Датчики |
| `VoiceCallService` | DBus (org.nemomobile.voicecall) | Звонки |
| `ContactService` | DBus (ru.auroraos.contacts1) | Контакты |

---

## Ограничения Aurora OS

1. **DBus сигналы** — могут требовать особой обработки на Aurora
2. **Права доступа** — некоторые DBus интерфейсы ограничены
3. **DConf CLI** — команда `dconf` должна быть доступна в системе
4. **Потокобезопасность** — DBus соединения не thread-safe, требуется мьютекс

---

## Зависимости

```toml
[dependencies]
dbus = "0.9"
serde = { version = "1", features = ["derive"] }
thiserror = "1"
```

---

## Следующие шаги

1. Реализовать `error.rs`
2. Реализовать `backends/dbus.rs`
3. Реализовать `backends/dconf.rs`
4. Реализовать `services/notifications.rs`
5. Реализовать `services/settings.rs`
6. Написать тесты
7. Документация (rustdoc)
