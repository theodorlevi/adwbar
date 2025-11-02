use relm4::gtk::prelude::*;
use niri_ipc::socket::Socket;
use niri_ipc::{Request, Response};
use relm4::gtk::Image;
use std::collections::BTreeMap;

/// Updates the window list container with current windows from niri
pub fn update_window_list(container: &relm4::gtk::Box) {
    match Socket::connect().and_then(|mut s| s.send(Request::Windows)) {
        Ok(Ok(Response::Windows(windows))) => {
            // Group windows by workspace
            let mut workspaces: BTreeMap<u64, Vec<(String, u64)>> = BTreeMap::new();

            for window in windows {
                let workspace_id = window.workspace_id.unwrap_or(0);
                let app_id = window
                    .app_id
                    .unwrap_or_else(|| "dialog-question-symbolic".to_string());
                let window_id = window.id;
                workspaces
                    .entry(workspace_id)
                    .or_insert_with(Vec::new)
                    .push((app_id, window_id));
            }

            // Clear existing widgets
            while let Some(child) = container.first_child() {
                container.remove(&child);
            }

            // Build new workspace sections
            for (workspace_id, windows) in workspaces {
                let workspace_box = create_workspace_section(workspace_id, windows);
                container.append(&workspace_box);
            }
        }
        Ok(_) => eprintln!("Unexpected response from niri"),
        Err(e) => eprintln!("Failed to get windows: {}", e),
    }
}

fn create_workspace_section(workspace_id: u64, windows: Vec<(String, u64)>) -> relm4::gtk::Box {
    let workspace_box = relm4::gtk::Box::new(relm4::gtk::Orientation::Horizontal, 3);
    workspace_box.add_css_class("workspace-section");

    // Add a workspace label
    let label = relm4::gtk::Label::new(Some(&workspace_id.to_string()));
    label.add_css_class("workspace-label");
    workspace_box.append(&label);

    // Add app icon buttons
    for (app_id, window_id) in windows {
        let button = create_window_button(&app_id, window_id);
        workspace_box.append(&button);
    }

    workspace_box
}

fn create_window_button(app_id: &str, window_id: u64) -> relm4::gtk::Button {
    let button = relm4::gtk::Button::new();
    button.add_css_class("flat");

    let icon = Image::builder()
        .icon_name(app_id)
        .pixel_size(16)
        .build();
    button.set_child(Some(&icon));

    // Focus window on click
    button.connect_clicked(move |_| {
        if let Ok(mut socket) = Socket::connect() {
            let _ = socket.send(Request::Action(niri_ipc::Action::FocusWindow {
                id: window_id,
            }));
        }
    });

    button
}
