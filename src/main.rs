mod config_window;
mod connectivity;
mod messages;
mod system_monitor;
mod window_list;

use chrono::Local;
use clap::Parser;
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use relm4::adw::glib;
use relm4::gtk::{Align, Image};
use relm4::prelude::*;

use connectivity::{ConnectivityStatus, read_bluetooth_status, read_wifi_status};
use messages::AppMessage;
use system_monitor::{read_cpu_usage, read_gpu_usage};
use window_list::update_window_list;

#[derive(Parser, Debug, Clone)]
#[command(name = "adwbar")]
#[command(about = "A status bar application", long_about = None)]
struct Args {
    /// Monitor name to display the bar on (e.g., HDMI-A-1, DP-1)
    #[arg(short, long)]
    monitor: Option<String>,
}

struct App {
    clock: String,
    window_list_container: gtk::Box,
    system_info: (String, String),
    wifi_status: ConnectivityStatus,
    bluetooth_status: ConnectivityStatus,
    config_window: Controller<config_window::ConfigWindow>,
}

#[relm4::component]
impl SimpleComponent for App {
    type Input = AppMessage;
    type Output = ();
    type Init = Args;

    view! {
        window = adw::ApplicationWindow {
            set_title: Some("Layer Shell with Adwaita"),
            set_default_size: (24, 24),
            add_css_class: "main-bar-window",

            adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    add_css_class: "main-container",
                    set_show_end_title_buttons: false,
                    set_show_start_title_buttons: false,

                    // window list container
                    #[local_ref]
                    pack_start = &window_list_container -> gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 5,
                        set_halign: Align::Start,
                        set_hexpand: true,
                    },

                    // clock container
                    #[wrap(Some)]
                    set_title_widget = &gtk::Button {
                        add_css_class: "clock-container",
                        add_css_class: "flat",
                        set_halign: Align::Center,
                        gtk::Label {
                            #[watch]
                            set_label: &model.clock,
                            add_css_class: "title-2",
                        }
                    },

                    // System info container
                    pack_end = &gtk::Box {
                        add_css_class: "system-info-container",
                        set_halign: Align::End,
                        set_hexpand: true,
                        set_spacing: 5,
                        gtk::Button {
                            add_css_class: "system-info-button",
                            add_css_class: "flat",
                            set_halign: Align::End,
                            connect_clicked[sender] => move |_| {
                                sender.input(AppMessage::SystemInfoClicked);
                            },
                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 8,
                                // System info
                                gtk::Label {
                                    #[watch]
                                    set_label: &format!("CPU: {} GPU: {}", model.system_info.0, model.system_info.1),
                                    add_css_class: "system-info-label",
                                },
                                // WiFi icon
                                Image {
                                    #[watch]
                                    set_icon_name: Some(if model.wifi_status.enabled {
                                        "network-wireless-signal-excellent-symbolic"
                                    } else {
                                        "network-wireless-disabled-symbolic"
                                    }),
                                    set_pixel_size: 16,
                                },
                                // Bluetooth icon
                                Image {
                                    #[watch]
                                    set_icon_name: Some(if model.bluetooth_status.enabled {
                                        "bluetooth-active-symbolic"
                                    } else {
                                        "bluetooth-disabled-symbolic"
                                    }),
                                    set_pixel_size: 16,
                                },
                            }
                        }
                    }
                }
            }
        }
    }

    fn init(
        args: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Initialize layer shell
        root.init_layer_shell();
        root.set_layer(Layer::Top);
        root.auto_exclusive_zone_enable();
        root.set_anchor(Edge::Top, true);
        root.set_anchor(Edge::Left, true);
        root.set_anchor(Edge::Right, true);
        root.set_height_request(24);

        // Set monitor if specified
        if let Some(monitor_name) = args.monitor {
            let display = gtk::gdk::Display::default().expect("Could not get default display");
            let monitor_list = display.monitors();
            let num_monitors = monitor_list.n_items();

            let mut found = false;
            for i in 0..num_monitors {
                if let Some(monitor_obj) = monitor_list.item(i) {
                    let monitor = monitor_obj
                        .downcast::<gtk::gdk::Monitor>()
                        .expect("Failed to downcast to Monitor");
                    if let Some(connector) = monitor.connector() {
                        if connector.as_str() == monitor_name {
                            root.set_monitor(Some(&monitor));
                            found = true;
                            break;
                        }
                    }
                }
            }

            if !found {
                eprintln!(
                    "Warning: Monitor '{}' not found, using default",
                    monitor_name
                );
                eprintln!("Available monitors:");
                for i in 0..num_monitors {
                    if let Some(monitor_obj) = monitor_list.item(i) {
                        let monitor = monitor_obj
                            .downcast::<gtk::gdk::Monitor>()
                            .expect("Failed to downcast to Monitor");
                        if let Some(connector) = monitor.connector() {
                            eprintln!("  - {}", connector);
                        }
                    }
                }
            }
        }

        // Load CSS
        let css_provider = gtk::CssProvider::new();
        css_provider.load_from_bytes(&glib::Bytes::from(include_bytes!("style.css")));
        gtk::style_context_add_provider_for_display(
            &gtk::gdk::Display::default().unwrap(),
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // Initialize model
        let clock = Local::now().format("%H:%M").to_string();
        let window_list_container = gtk::Box::new(gtk::Orientation::Horizontal, 5);

        let config_window = config_window::ConfigWindow::builder()
            .transient_for(&root)
            .launch(())
            .detach();

        let model = App {
            clock,
            window_list_container: window_list_container.clone(),
            system_info: (String::new(), String::new()),
            wifi_status: ConnectivityStatus::unknown(),
            bluetooth_status: ConnectivityStatus::unknown(),
            config_window,
        };

        let widgets = view_output!();

        setup_timers(&sender);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            AppMessage::ClockUpdate => {
                self.clock = Local::now().format("%H:%M").to_string();
            }
            AppMessage::WindowListUpdate => {
                update_window_list(&self.window_list_container);
            }
            AppMessage::SystemInfoUpdate => {
                self.system_info = (read_cpu_usage(), read_gpu_usage());
            }
            AppMessage::ConnectivityUpdate => {
                self.wifi_status = read_wifi_status();
                self.bluetooth_status = read_bluetooth_status();
            }
            AppMessage::SystemInfoClicked => {
                self.config_window.widget().set_visible(true);
            }
        }
    }
}

fn setup_timers(sender: &ComponentSender<App>) {
    // Update clock
    let sender_clone = sender.clone();
    glib::timeout_add_seconds_local(1, move || {
        sender_clone.input(AppMessage::ClockUpdate);
        glib::ControlFlow::Continue
    });

    // Update window
    let sender_clone = sender.clone();
    glib::timeout_add_seconds_local(1, move || {
        sender_clone.input(AppMessage::WindowListUpdate);
        glib::ControlFlow::Continue
    });

    // Update system info
    let sender_clone = sender.clone();
    glib::timeout_add_seconds_local(1, move || {
        sender_clone.input(AppMessage::SystemInfoUpdate);
        glib::ControlFlow::Continue
    });

    // Update connectivity status
    let sender_clone = sender.clone();
    glib::timeout_add_seconds_local(1, move || {
        sender_clone.input(AppMessage::ConnectivityUpdate);
        glib::ControlFlow::Continue
    });
}

fn main() {
    // Parse arguments before GTK initializes
    let args = Args::parse();

    // Create app with no GTK arguments
    let app = RelmApp::new("me.bofusland.adwbar").with_args(Vec::<String>::new());

    app.run::<App>(args);
}
