use zbus::blocking::Connection;

/// WiFi and Bluetooth status
#[derive(Debug, Clone)]
pub struct ConnectivityStatus {
    pub enabled: bool,
    pub _status: String,
}

impl ConnectivityStatus {
    pub fn new(enabled: bool, _status: String) -> Self {
        Self { enabled, _status }
    }

    pub fn unknown() -> Self {
        Self {
            enabled: false,
            _status: "Unknown".to_string(),
        }
    }
}

/// Checks WiFi status via NetworkManager DBus
pub fn read_wifi_status() -> ConnectivityStatus {
    match Connection::system() {
        Ok(conn) => {
            let proxy = zbus::blocking::Proxy::new(
                &conn,
                "org.freedesktop.NetworkManager",
                "/org/freedesktop/NetworkManager",
                "org.freedesktop.NetworkManager",
            );

            if let Ok(proxy) = proxy {
                if let Ok(wifi_enabled) = proxy.get_property::<bool>("WirelessEnabled") {
                    if let Ok(active_connections) = proxy
                        .get_property::<Vec<zbus::zvariant::OwnedObjectPath>>("ActiveConnections")
                    {
                        if !active_connections.is_empty() && wifi_enabled {
                            return ConnectivityStatus::new(true, "Connected".to_string());
                        }
                    }

                    if wifi_enabled {
                        return ConnectivityStatus::new(true, "Enabled".to_string());
                    } else {
                        return ConnectivityStatus::new(false, "Disabled".to_string());
                    }
                }
            }
            ConnectivityStatus::unknown()
        }
        Err(_) => ConnectivityStatus::new(false, "N/A".to_string()),
    }
}

/// Checks Bluetooth status via BlueZ DBus
pub fn read_bluetooth_status() -> ConnectivityStatus {
    match Connection::system() {
        Ok(conn) => {
            let proxy = zbus::blocking::Proxy::new(
                &conn,
                "org.bluez",
                "/org/bluez/hci0",
                "org.bluez.Adapter1",
            );

            if let Ok(proxy) = proxy {
                if let Ok(powered) = proxy.get_property::<bool>("Powered") {
                    if powered {
                        return ConnectivityStatus::new(true, "Enabled".to_string());
                    } else {
                        return ConnectivityStatus::new(false, "Disabled".to_string());
                    }
                }
            }
            ConnectivityStatus::unknown()
        }
        Err(_) => ConnectivityStatus::new(false, "N/A".to_string()),
    }
}
