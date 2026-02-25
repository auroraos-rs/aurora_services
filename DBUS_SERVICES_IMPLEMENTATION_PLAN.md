# D-Bus Services Implementation Plan

## Status Overview

| Service | Status | Priority |
|---------|--------|----------|
| Notification Service | ✅ Completed | 1 |
| DeviceInfo Service | ✅ Features provider completed | 2 |
| Bluetooth Service | Not started | 3 |
| NFC Service | Not started | 4 |
| Voice Call Service | Not started | 5 |
| Location Service | Not started | 6 |

**Note**: Contacts Service was removed - the `ru.auroraos.contacts1` D-Bus service is not available on devices.

---

## 1. Notification Service (`org.freedesktop.Notifications`)

**Status**: ✅ **COMPLETED**

**Implementation** (`aurora_services/src/services/notifications.rs`):
- `Notification` struct with all standard fields
- `NotificationService` with methods:
  - `notify()` - Send notification
  - `close()` - Close notification
  - `get_capabilities()` - Get server capabilities
  - `get_server_info()` - Get server information
- `NotificationBuilder` - Builder pattern for creating notifications
- `VariantValue` enum for hints (Int, UInt, Double, String, Bool, Byte, ByteArray)
- `ServerInfo` struct for server information

**Bus**: Session  
**Service Name**: `org.freedesktop.Notifications`  
**Object Path**: `/org/freedesktop/Notifications`

**Methods**:
- `Notify(appName, replacesId, appIcon, summary, body, actions, hints, expireTimeout) -> uint`
- `CloseNotification(id)` -> Closes a notification
- `GetCapabilities() -> array` - Get server capabilities
- `GetNotifications(appName) -> array` - Get notifications by app
- `GetNotificationsByCategory(category) -> array` - Get notifications by category
- `GetServerInformation() -> (name, vendor, version, spec_version)`

**Signals**:
- `NotificationClosed(id, reason)` - Notification was closed
- `ActionInvoked(id, action)` - User action triggered
- `InputTextSet(id, text)` - Text input from notification

**Aurora-specific extensions (Nemo hints)**:
- `x-nemo-item-count` - Number of items
- `x-nemo-priority` - Priority level (default 50)
- `x-nemo-timestamp` - Event timestamp
- `x-nemo-preview-body` - Preview text body
- `x-nemo-preview-summary` - Preview text summary
- `x-nemo-remote-action-*` - Remote D-Bus action
- `x-nemo-visibility` - Visibility (public/private/secret)
- `x-nemo-feedback` - Feedback token
- `x-nemo-display-on` - Turn on display
- `x-nemo-user-removable` - User can remove

**Capabilities**:
- `persistence` - Keep notifications in event view
- `body` - Support notification body text
- `actions` - Support notification actions
- `sound` - Support sound hints
- `x-nemo-remote-actions` - Support remote actions

**Remaining Work**:
- Add signal handling (`NotificationClosed`, `ActionInvoked`, `InputTextSet`)
- Implement `GetNotifications()` and `GetNotificationsByCategory()` methods
- Add full Aurora-specific Nemo hints support

---

## 2. Bluetooth Service (`org.bluez`)

**Status**: Not started

**Bus**: System  
**Service Name**: `org.bluez`  
**Object Path**: `/org/bluez/hci0`

**Key Interfaces**:

### Adapter1 (`org.bluez.Adapter1`)
**Properties**:
- `Address` (read) - Bluetooth address
- `AddressType` (read) - public/random
- `Name` (read) - Adapter name
- `Alias` (readwrite) - User-friendly name
- `Class` (read) - Device class
- `Powered` (readwrite) - Power state
- `Discoverable` (readwrite) - Discoverable mode
- `DiscoverableTimeout` (readwrite) - Timeout in seconds
- `Pairable` (readwrite) - Pairable mode
- `PairableTimeout` (readwrite) - Pairable timeout
- `Discovering` (read) - Currently discovering
- `UUIDs` (read) - Supported services

**Methods**:
- `StartDiscovery()` - Begin device discovery
- `StopDiscovery()` - Stop discovery
- `RemoveDevice(device)` - Remove paired device
- `SetDiscoveryFilter(filter)` - Set discovery filter
- `GetDiscoveryFilters()` - Get available filters
- `ConnectDevice(properties)` - Connect to device directly

### Device1 (`org.bluez.Device1`)
**Properties**:
- `Address`, `AddressType`, `Name`, `Alias`
- `Connected`, `Paired`, `Trusted`
- `ServicesResolved`, `Adapter`
- `UUIDs`, `Class`

**Methods**:
- `Connect()` - Connect to device
- `Disconnect()` - Disconnect
- `Pair()` - Initiate pairing
- `CancelPairing()` - Cancel pairing

### Media1 (`org.bluez.Media1`)
**Methods**:
- `RegisterEndpoint(endpoint, properties)`
- `UnregisterEndpoint(endpoint)`
- `RegisterPlayer(player, properties)`
- `UnregisterPlayer(player)`

### GattManager1 (`org.bluez.GattManager1`)
**Methods**:
- `RegisterApplication(application, options)`
- `UnregisterApplication(application)`

### BatteryManager1 (`org.bluez.BatteryManager1`)
**Methods**:
- `RegisterBatteryProvider(provider)`
- `UnregisterBatteryProvider(provider)`

### AgentManager1 (`org.bluez.AgentManager1`)
**Methods**:
- `RegisterAgent(agent, capability)`
- `UnregisterAgent(agent)`
- `RequestDefaultAgent(agent)`

---

## 3. NFC Service (`org.sailfishos.nfc.daemon`)

**Status**: Not started

**Bus**: System  
**Service Name**: `org.sailfishos.nfc.daemon`  
**Object Path**: `/nfc0`

**Key Interfaces**:

### Adapter (`org.sailfishos.nfc.Adapter`)
**Properties**:
- `Enabled` (readwrite) - NFC enabled state
- `Powered` (readwrite) - NFC powered state
- `SupportedModes` (read) - Supported NFC modes
- `Mode` (readwrite) - Current NFC mode
- `TargetPresent` (read) - Tag present
- `Tags` (read) - Available tags
- `Peers` (read) - Available peers

**Methods**:
- `GetAll()` -> Get all properties
- `GetEnabled()`, `SetEnabled(bool)`
- `GetPowered()`, `SetPowered(bool)`
- `GetMode()`, `SetMode(uint)`
- `GetTags()` -> Get tag object paths
- `GetPeers()` -> Get peer object paths

### Daemon (`org.freedesktop.DBus.ObjectManager`)
**Methods**:
- `GetManagedObjects()` -> Get all managed objects

**Signals**:
- `InterfacesAdded(object, interfaces)`
- `InterfacesRemoved(object, interfaces)`

### Standard NFC Interfaces
- `org.neard.Adapter` - Standard NFC adapter control
- `org.neard.Tag` - NFC tag operations
ard.NDEF` - NDEF read/write operations
- `org- `org.ne.neard.IsoDep` - ISO-DEP (ISO 14443-4) operations
- `org.neard.TagType1` - Type 1 tags
- `org.neard.TagType2` - Type 2 tags
- `org.neard.TagType3` - Type 3 tags
- `org.neard.TagType4` - Type 4 tags
- `org.neard.Peer` - P2P communication via LLCP

**Methods (Tag/NDEF)**:
- `ReadNDEF()` -> Read NDEF data
- `WriteNDEF(data)` -> Write NDEF data
- `StartPollLoop(technology)` - Begin tag polling
- `StopPollLoop()` - Stop polling

**Required Permission**: `NFC`

---

## 4. Voice Call Service (`ru.auroraos.Call`)

**Status**: Not started

**Bus**: Session  
**Service Name**: `ru.auroraos.Call.Service1`  
**Object Path**: Custom (per application)

**Key Interfaces**:

### Service1 (`ru.auroraos.Call.Service1`)
**Methods**:
- `RegisterCallManager(objectPath)` - Register call manager
- `UnregisterCallManager(objectPath)` - Unregister

### Call1 (`ru.auroraos.Call.Call1`)
**Properties**:
- `Status` (readwrite) - Call status (see below)
- `Direction` (read) - incoming/outgoing
- `RemoteName` (read) - Caller name/number
- `RemoteId` (read) - Caller ID
- `StartTime` (read) - Call start timestamp
- `Duration` (read) - Call duration
- `IsEmergency` (read) - Emergency call flag
- `IsConference` (read) - Conference call flag
- `IsOnHold` (read) - Held call flag
- `Incoming` (read) - Incoming call flag
- `Multiparty` (read) - Multiparty call flag

**Methods**:
- `Answer()` - Answer incoming call
- `Hangup()` - End call
- `Hold()` - Hold call
- `Resume()` - Resume held call
- `Deflect(number)` - Deflect to another number
- `SendTones(tones)` - Send DTMF tones

**Call Status Codes**:
- `1` - Dialing (outgoing, not connected)
- `2` - Alerting (ringing)
- `3` - Ringing (incoming, not answered)
- `4` - Active (call in progress)
- `5` - Held (on hold)
- `6` - Disconnected (ended)

### DTMF1 (`ru.auroraos.Call.DTMF1`)
**Methods**:
- `SendTones(tones)` - Send DTMF tone sequence

**Signals**:
- `ToneReceived(tone)` - DTMF tone received

### AudioControl1 (`ru.auroraos.Call.AudioControl1`)
**Methods**:
- `SetOutputDevice(device)` - Set audio output (earpiece/speaker/bluetooth)
- `SetInputDevice(device)` - Set audio input

**Signals**:
- `OutputDeviceChanged(device)`
- `InputDeviceChanged(device)`

### IconProvider1 (`ru.auroraos.Call.IconProvider1`)
**Methods**:
- `GetIcon()` -> Get icon data

**Key Features**:
- Registration via `org.freedesktop.DBus.ObjectManager`
- Signal-based call state changes (`InterfacesAdded`, `InterfacesRemoved`)
- Automatic audio routing and screen management
- Echo cancellation support

**Required Permission**: `Call`

---

## 5. DeviceInfo Service (`ru.omp.deviceinfo`)

**Status**: ✅ **Features provider completed** (Storages and SIM pending)

**Bus**: System  
**Service Name**: `ru.omp.deviceinfo`  
**Object Path**: `/ru/omp/deviceinfo/*`

### Implemented: Features Provider

**Object Path**: `/ru/omp/deviceinfo/Features`  
**Interface**: `ru.omp.deviceinfo.Features`

**Implementation** (`aurora_services/src/services/device_info/features.rs`):
- `DeviceInfoService` struct with `get_features()` method
- `DeviceFeatures` struct with all device properties:
  - Device: model, serial number, OS version, OS type, certified, emulated
  - CPU: model, cores, clock speed, per-core speeds
  - Memory: RAM total/free
  - Screen: resolution
  - Camera: main and frontal resolutions
  - Battery: charge percentage
  - Hardware: Bluetooth, NFC, GNSS, WLAN
  - Localization: locale, timezone, locales, fonts

### Pending: Storages Provider

**Object Path**: `/ru/omp/deviceinfo/Storages`  
**Interface**: `ru.omp.deviceinfo.Storages`

**Methods**:
- `getInternalStorageInfo()` - Internal flash memory
- `getExternalStorageInfo()` - External storage (deprecated)
- `getInternalUserPartitionInfo()` - User partition
- `getExternalStorageDrivesInfo()` - External drives
- `getSystemStorageInfo()` - System storage info
- `getUsersQuotaInfo()` - User quotas

### Pending: SIM Provider

**Object Path**: `/ru/omp/deviceinfo/SIM`  
**Interface**: `ru.omp.deviceinfo.SIM`

**Methods**:
- `getSimCardsInfo()` - Get SIM card information

**Required Permission**: `DeviceInfo` in desktop file

---

## 6. Location Service (`ru.omp.LocationService`)

**Status**: Not started

**Bus**: System  
**Service Name**: `ru.omp.LocationService`  
**Object Path**: `/ru/omp/LocationService`

**Key Interfaces**:

### CacheResetter (`ru.omp.LocationService.CacheResetter`)
**Methods**:
- `Reset() -> bool` - Reset location cache

### Conf (`ru.omp.LocationService.Conf`)
**Methods**:
- `AgnssProxyServerGetAll() -> (enabled, address, port, updateInterval, tls, verbose)`
- `SetAgnssProxyServer(address, port)` - Set AGPS server
- `GetAgnssProxyServer() -> (address, port)` - Get AGPS server
- `SetAgnssProxyUpdateInterval(interval)` - Set update interval
- `GetAgnssProxyUpdateInterval() -> int` - Get update interval
- `SetAgnssProxyEnabled(enabled)` - Enable/disable AGPS
- `GetAgnssProxyEnabled() -> bool` - Get AGPS enabled state
- `SetAgnssProxyTlsEnabled(enabled)` - Enable TLS
- `GetAgnssProxyTlsEnabled() -> bool` - Get TLS state
- `SetAgnssProxySslPinning(enabled)` - Enable SSL pinning
- `SetAgnssProxySslPinningKey(key)` - Set SSL pinning key
- `SetAgnssProxyVerbose(enabled)` - Enable verbose logging
- `GetAgnssProxyVerbose() -> bool` - Get verbose state
- `GetUserAgnssProxyConfAvailable() -> bool` - Check if user config available

**Configuration**:
- Settings stored in `/etc/location/agnss-proxy.conf`
- A-GNSS (Assisted GPS) for faster positioning
- SUPL server configuration

**Note**: Requires MDM-signed application for full API access

---

## Implementation Dependencies

| Service | Dependencies |
|---------|--------------|
| Notification | `dbus`, `serde` (completed) |
| Bluetooth | `dbus`, `serde` |
| NFC | `dbus`, `serde`, NFC hardware |
| Voice Call | `dbus`, `serde`, telephony backend |
| DeviceInfo | `dbus`, `serde`, hardware info |
| Location | `dbus`, `serde`, GPS/network providers |

---

## Recommended Implementation Order

1. **Notification Service** - Completed
2. **Bluetooth Service** - Start with Adapter interface
3. **DeviceInfo Service** - Simple read-only interface
4. **NFC Service** - Medium complexity, hardware-dependent
5. **Voice Call Service** - Complex state machine
6. **Location Service** - Configuration-focused
