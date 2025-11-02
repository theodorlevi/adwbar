use crate::connectivity::{ConnectivityStatus, read_bluetooth_status, read_wifi_status};
use crate::messages::ConfMessage;
use crate::system_monitor::{read_cpu_usage, read_gpu_usage};
use gtk4_layer_shell::{Layer, LayerShell};
use relm4::adw::glib;
use relm4::adw::prelude::*;
use relm4::prelude::*;
use zbus::blocking::Connection;

pub struct ConfigWindow {
    wifi_status: ConnectivityStatus,
    bluetooth_status: ConnectivityStatus,
    cpu_usage: String,
    gpu_usage: String,
}

#[relm4::component(pub)]
impl SimpleComponent for ConfigWindow {
    type Init = ();
    type Input = ConfMessage;
    type Output = ();

    view! {
        adw::ApplicationWindow {
            set_default_size: (400, 500),
            set_title: Some("Control Center"),
            set_hide_on_close: true,
            add_css_class: "config-window",

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 0,

                adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &gtk::Label {
                        set_css_classes: &["flat"],
                        set_label: "Control Center",
                    }
                },

                gtk::ScrolledWindow {
                    set_vexpand: true,
                    set_hexpand: true,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 12,
                        set_margin_all: 12,

                        // System Information Section
                        adw::PreferencesGroup {
                            set_title: "System Information",

                            adw::ActionRow {
                                set_title: "CPU Usage",
                                add_suffix = &gtk::Label {
                                    #[watch]
                                    set_label: &model.cpu_usage,
                                    add_css_class: "dim-label",
                                }
                            },

                            adw::ActionRow {
                                set_title: "GPU Usage",
                                add_suffix = &gtk::Label {
                                    #[watch]
                                    set_label: &model.gpu_usage,
                                    add_css_class: "dim-label",
                                }
                            },
                        },

                        // Connectivity Section
                        adw::PreferencesGroup {
                            set_title: "Connectivity",

                            adw::ActionRow {
                                set_title: "WiFi",
                                set_subtitle: &model.wifi_status._status,
                                add_suffix = &gtk::Switch {
                                    set_valign: gtk::Align::Center,
                                    #[watch]
                                    set_active: model.wifi_status.enabled,
                                    connect_state_set[sender] => move |_, enabled| {
                                        sender.input(ConfMessage::ToggleWifi(enabled));
                                        glib::Propagation::Proceed
                                    }
                                },
                            },

                            adw::ActionRow {
                                set_title: "Bluetooth",
                                set_subtitle: &model.bluetooth_status._status,
                                add_suffix = &gtk::Switch {
                                    set_valign: gtk::Align::Center,
                                    #[watch]
                                    set_active: model.bluetooth_status.enabled,
                                    connect_state_set[sender] => move |_, enabled| {
                                        sender.input(ConfMessage::ToggleBluetooth(enabled));
                                        glib::Propagation::Proceed
                                    }
                                },
                            },
                        },
                    }
                }
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Initialize layer shell
        root.init_layer_shell();
        root.set_layer(Layer::Overlay);
        root.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);

        // Load CSS for config window
        let css_provider = relm4::gtk::CssProvider::new();
        css_provider.load_from_bytes(&glib::Bytes::from(include_bytes!("config_style.css")));
        relm4::gtk::style_context_add_provider_for_display(
            &relm4::gtk::gdk::Display::default().unwrap(),
            &css_provider,
            relm4::gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let model = Self {
            wifi_status: read_wifi_status(),
            bluetooth_status: read_bluetooth_status(),
            cpu_usage: read_cpu_usage(),
            gpu_usage: read_gpu_usage(),
        };

        let widgets = view_output!();

        // Start hidden
        root.set_visible(false);

        // Setup periodic updates
        let sender_clone = sender.clone();
        glib::timeout_add_seconds_local(2, move || {
            sender_clone.input(ConfMessage::UpdateStatus);
            glib::ControlFlow::Continue
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            ConfMessage::UpdateStatus => {
                self.wifi_status = read_wifi_status();
                self.bluetooth_status = read_bluetooth_status();
                self.cpu_usage = read_cpu_usage();
                self.gpu_usage = read_gpu_usage();
            }
            ConfMessage::ToggleWifi(enabled) => {
                if let Ok(conn) = Connection::system() {
                    if let Ok(proxy) = zbus::blocking::Proxy::new(
                        &conn,
                        "org.freedesktop.NetworkManager",
                        "/org/freedesktop/NetworkManager",
                        "org.freedesktop.NetworkManager",
                    ) {
                        let _ = proxy.set_property("WirelessEnabled", enabled);
                    }
                }
                self.wifi_status = read_wifi_status();
            }
            ConfMessage::ToggleBluetooth(enabled) => {
                if let Ok(conn) = Connection::system() {
                    if let Ok(proxy) = zbus::blocking::Proxy::new(
                        &conn,
                        "org.bluez",
                        "/org/bluez/hci0",
                        "org.bluez.Adapter1",
                    ) {
                        let _ = proxy.set_property("Powered", enabled);
                    }
                }
                self.bluetooth_status = read_bluetooth_status();
            }
        }
    }
}
