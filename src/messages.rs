/// Messages for the main application component
#[derive(Debug)]
pub enum AppMessage {
    ClockUpdate,
    WindowListUpdate,
    SystemInfoUpdate,
    ConnectivityUpdate,
    SystemInfoClicked,
}

/// messages for config window component
#[derive(Debug)]
pub enum ConfMessage {
    UpdateStatus,
    ToggleWifi(bool),
    ToggleBluetooth(bool),
}
