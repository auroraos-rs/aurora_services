# Алгоритм сбора D-Bus данных - Aurora OS

## Файлы результатов
- `dbus_system.txt` — системная шина
- `dbus_session.txt` — сессионная шина

---

## Фаза 0: Подготовка

```
Создать dbus_system.txt:
# DBus System Bus Dump - Aurora OS
# Generated: <дата>
# Device: 192.168.2.15 (ssh defaultuser@192.168.2.15)
================================================================================

Создать dbus_session.txt с аналогичным заголовком
```

---

## Фаза 1: System Bus (по одному сервису)

### Шаг 1.1: Получить список сервисов
```bash
dbus-send --system --dest=org.freedesktop.DBus --print-reply \
  /org/freedesktop/DBus org.freedesktop.DBus.ListNames
```

### Шаг 1.2: Отфильтровать
Убрать уникальные имена (:1.xxx), оставить только well-known names.

### Шаг 1.3: Для каждого сервиса

```
Вывести прогресс: "Processing SERVICE X/Y: <name>"

1.3.1 ЗАПИСАТЬ заголовок:
      ================================================================================
      SERVICE <N>/<TOTAL>: <имя>
      ================================================================================
      Bus: system

1.3.2 ИНТРОСПЕКЦИЯ корневого пути "/":
      dbus-send --system --dest=<SERVICE> --print-reply / \
        org.freedesktop.DBus.Introspectable.Introspect
      
      - Успех → парсить XML, ЗАПИСАТЬ результат
      - Ошибка "Access denied" → записать "(Access denied at /)"
      - Ошибка/краш → записать "(Introspection failed at /)"
      - Пропустить краш-сервис, перейти к следующему

1.3.3 ИНТРОСПЕКЦИЯ dot-to-slash пути (ВСЕГДА):
      Преобразовать: org.bluez.obex → /org/bluez/obex
      dbus-send --system --dest=<SERVICE> --print-reply <PATH> \
        org.freedesktop.DBus.Introspectable.Introspect
      
      - Успех → парсить XML, ЗАПИСАТЬ результат
      - Ошибка → записать "(No introspection at <path>)"
      - Если путь совпадает с "/" → пропустить (уже обработан в 1.3.2)

1.3.4 РЕКУРСИВНЫЙ ОБХОД дочерних узлов:
      Из XML (из 1.3.2 и/или 1.3.3) найти все <node name="X">
      Для каждого уникального пути:
        - Introspect <path>
        - Записать результат

1.3.5 ПРОВЕРКА ObjectManager:
      Если интерфейс org.freedesktop.DBus.ObjectManager найден:
        dbus-send --system --dest=<SERVICE> --print-reply <PATH> \
          org.freedesktop.DBus.ObjectManager.GetManagedObjects
        - Интроспектировать все пути из ответа

1.3.6 ЗАПИСАТЬ всё в файл (см. формат ниже)

1.3.7 Разделитель перед следующим сервисом
```

---

## Фаза 2: Session Bus

Аналогично Фазе 1, но:
- `--session` вместо `--system`
- Запись в `dbus_session.txt`

---

## Методы получения путей

| Метод | Описание |
|-------|----------|
| `org.freedesktop.DBus.Introspectable.Introspect` | Возвращает XML с дочерними узлами `<node name="...">` |
| `org.freedesktop.DBus.ObjectManager.GetManagedObjects` | Возвращает `a{oa{sa{sv}}}` — все пути сразу. Используется в org.bluez, UDisks2, neard |

---

## Формат выходного файла

```
================================================================================
SERVICE 1/52: org.bluez.obex
================================================================================
Bus: system

--- Introspection at / ---
(No child nodes)

--- Introspection at /org/bluez/obex ---
Path: /org/bluez/obex

org.bluez.obex.AgentManager1:
  Methods:
    - RegisterAgent(o agent)
    - UnregisterAgent(o agent)
  Properties:
    - None
  Signals:
    - None

org.bluez.obex.Client1:
  Methods:
    - CreateSession(s destination, a{sv} args) -> o session
    - RemoveSession(o session)
  Properties:
    - None
  Signals:
    - None

--------------------------------------------------------------------------------

================================================================================
SERVICE 2/52: org.ofono
================================================================================
Bus: system

--- Introspection at / ---
Path: /

org.ofono.Manager:
  Methods:
    - GetModems() -> a(oa{sv})
  Properties:
    - None
  Signals:
    - ModemAdded(o path, a{sv} properties)
    - ModemRemoved(o path)

Child nodes: /ril_0, /ril_1

--- Introspection at /org/ofono ---
(Introspection failed at /org/ofono)

--- Recursively introspecting child nodes ---

Path: /ril_0
org.ofono.Modem:
  Properties:
    - Powered: b (readwrite)
    - Online: b (readwrite)
    - Interfaces: as (read)
  Methods:
    - ...
  Signals:
    - ...

--------------------------------------------------------------------------------
```

---

## Формат записи данных

### Properties
```
  Properties:
    - <имя>: <тип> (read | readwrite | write)
```

### Methods (подробный формат)
```
  Methods:
    - <имя>(<тип1> arg1, <тип2> arg2, ...) -> <тип результата>
    - GetAll() -> i version, b enabled, b powered, u supported_modes, u mode, b target_present, ao tags
```

### Signals (с сигнатурами аргументов)
```
  Signals:
    - <имя>(<тип1> arg1, <тип2> arg2, ...)
    - EnabledChanged(b enabled)
    - TagsChanged(ao tags)
```

---

## Итоговые секции (в конце каждого файла)

```
================================================================================
SERVICES WITH LIMITED ACCESS
================================================================================
<список сервисов с Access denied / Introspection failed>

================================================================================
SUMMARY
================================================================================
Total: <N>
With full introspection: <N>
Limited access: <N>
Introspection failed: <N>
```

---

## Порядок выполнения (для одного сервиса)

```
1. ListNames (один раз в начале)
2. Introspect "/"
3. Introspect dot-to-slash path (ВСЕГДА)
4. Рекурсивно обойти дочерние узлы из обоих XML
5. Если есть ObjectManager: GetManagedObjects, интроспектировать все пути
6. Записать результат в файл
7. Следующий сервис
```

---

## D-Bus типы данных

| Сигнатура | Тип |
|-----------|-----|
| s | string |
| b | boolean |
| i | int32 |
| u | uint32 |
| x | int64 |
| t | uint64 |
| d | double |
| o | object path |
| v | variant |
| as | array of strings |
| ao | array of object paths |
| a{sv} | dict string→variant |
| a(oa{sv}) | array of (path, dict) |
