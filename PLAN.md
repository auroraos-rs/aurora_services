# План: Библиотека сервисов для Aurora OS

## Цель
Создать библиотеку для чтения и записи системных конфигураций Aurora OS (dconf + DBus).

## Этапы

### 1. Сбор информации о DBus интерфейсах
- [x] Подключиться к устройству по SSH
- [x] Получить список всех доступных DBus сервисов (системная и сессионная шины)
- [x] Для каждого сервиса собрать:
  - Имя сервиса
  - Пути объектов
  - Интерфейсы
  - Методы
  - Свойства
- [x] Сохранить результат в файлы `dbus_system.txt`, `dbus_session.txt`

### 2. Анализ собранной информации
- [x] Изучить dconf_dump.txt (уже собран)
- [x] Изучить dbus_system.txt, dbus_session.txt
- [x] Определить ключевые интерфейсы для работы с конфигурациями

### 3. Проектирование библиотеки
- [x] Определить архитектуру (backends + services)
- [x] Выбрать язык программирования и инструменты (Rust, dbus crate)
- [x] Написать спецификацию (SPEC.md)
- [x] Спроектировать API для работы с dconf
- [x] Спроектировать API для работы с DBus

### 4. Реализация
- [x] Реализовать `error.rs` — типы ошибок
- [x] Реализовать `backends/dbus.rs` — DBus backend
- [x] Реализовать `backends/dconf.rs` — DConf backend (чтение из txt файлов)
- [x] Реализовать `services/notifications.rs` — сервис уведомлений
- [x] Реализовать `services/settings.rs` — сервис настроек
- [x] Добавить `FontSettings` для настроек шрифтов

### 5. Тестирование
- [x] Создать демо-приложение `aurora_services_demo` (egui GUI)
- [x] Notifications работают на устройстве
- [ ] Протестировать DConf чтение из песочницы

### 6. Документация
- [ ] Описать использование библиотеки (rustdoc)
- [ ] Привести примеры кода

### 7. Дополнительные сервисы (будущее)
- [ ] BluetoothService (org.bluez)
- [ ] UsbService (com.meego.usb_moded)
- [ ] SensorService (com.nokia.SensorService)
- [ ] VoiceCallService (org.nemomobile.voicecall)
- [ ] ContactService (ru.auroraos.contacts1)

---

## Структура проекта

```
aurora_services/
├── Cargo.toml          # Workspace root
├── Cross.toml          # Cross-compilation config
├── aarch64_build.sh    # Build script for aarch64
├── arm_build.sh        # Build script for armv7
├── aurora_services/    # Library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── error.rs
│       ├── backends/
│       │   ├── mod.rs
│       │   ├── dbus.rs
│       │   └── dconf.rs
│       └── services/
│           ├── mod.rs
│           ├── notifications.rs
│           └── settings.rs
└── aurora_services_demo/  # GUI demo app
    ├── Cargo.toml
    ├── src/main.rs
    └── rpm/
```

---

## Дата начала: 2026-02-21

## Статус: Этап 5 — Тестирование на устройстве
