use gtk4::{Application, ApplicationWindow};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use toml::map::Map;
use toml::value::Value;

use gtk4::prelude::*;
use x11rb::connection::Connection;
use x11rb::errors::ReplyError;
use x11rb::protocol::xproto::{
    AtomEnum, ChangeWindowAttributesAux, ConfigureWindowAux, ConnectionExt, PropMode, StackMode,
};
use x11rb::rust_connection::RustConnection;

use crate::utils::DisplayBackend;

#[derive(Clone)]
pub struct WindowDescriptor {
    pub layer: Layer,

    pub margin_left: i32,
    pub margin_right: i32,
    pub margin_top: i32,
    pub margin_bottom: i32,

    pub anchor_left: bool,
    pub anchor_right: bool,
    pub anchor_top: bool,
    pub anchor_bottom: bool,

    pub exclusive: bool,
    pub keyboard_mode: KeyboardMode,

    pub visible_on_start: bool,
    pub namespace: String,
}

pub fn string_to_layer(str: &str) -> Layer {
    match str {
        "background" | "Background" => Layer::Background,
        "bottom" | "Bottom" => Layer::Bottom,
        "top" | "Top" => Layer::Top,
        "overlay" | "Overlay" => Layer::Overlay,
        _ => Layer::Overlay,
    }
}

impl Default for WindowDescriptor {
    fn default() -> Self {
        WindowDescriptor {
            layer: Layer::Overlay,

            margin_left: 0,
            margin_right: 0,
            margin_top: 0,
            margin_bottom: 0,

            anchor_left: false,
            anchor_right: false,
            anchor_top: false,
            anchor_bottom: false,

            exclusive: false,
            keyboard_mode: KeyboardMode::None,

            visible_on_start: true,
            namespace: "dvvidget".into(),
        }
    }
}

impl WindowDescriptor {
    pub fn from_toml(
        toml: &Map<String, Value>,
        key: &str,
        default: WindowDescriptor,
    ) -> WindowDescriptor {
        let mut result = default;

        let inner = if let Some(outer) = toml.get(key) {
            if let Some(val) = outer.get("window") {
                val
            } else {
                return result;
            }
        } else {
            return result;
        };

        if let Some(Value::String(val)) = inner.get("layer") {
            result.layer = string_to_layer(val);
        }

        if let Some(Value::Integer(val)) = inner.get("margin_left") {
            result.margin_left = *val as i32;
        }

        if let Some(Value::Integer(val)) = inner.get("margin_right") {
            result.margin_right = *val as i32;
        }

        if let Some(Value::Integer(val)) = inner.get("margin_top") {
            result.margin_top = *val as i32;
        }

        if let Some(Value::Integer(val)) = inner.get("margin_bottom") {
            result.margin_bottom = *val as i32;
        }

        if let Some(Value::Boolean(val)) = inner.get("anchor_left") {
            result.anchor_left = *val;
        }

        if let Some(Value::Boolean(val)) = inner.get("anchor_right") {
            result.anchor_right = *val;
        }

        if let Some(Value::Boolean(val)) = inner.get("anchor_top") {
            result.anchor_top = *val;
        }

        if let Some(Value::Boolean(val)) = inner.get("anchor_bottom") {
            result.anchor_bottom = *val;
        }

        if let Some(Value::Boolean(val)) = inner.get("exclusive") {
            result.exclusive = *val;
        }

        if let Some(Value::Boolean(val)) = inner.get("visible_on_start") {
            result.visible_on_start = *val;
        }

        result
    }
}

fn wayland_window(app: &Application, descriptor: &WindowDescriptor) -> ApplicationWindow {
    // Create a normal GTK window however you like
    let window = gtk4::ApplicationWindow::new(app);

    // Before the window is first realized, set it up to be a layer surface
    window.init_layer_shell();

    // Display above normal windows
    window.set_layer(descriptor.layer);

    window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);

    // Push other windows out of the way
    if descriptor.exclusive {
        window.auto_exclusive_zone_enable();
    }

    // The margins are the gaps around the window's edges
    // Margins and anchors can be set like this...
    window.set_margin(Edge::Left, descriptor.margin_left);
    window.set_margin(Edge::Right, descriptor.margin_right);
    window.set_margin(Edge::Top, descriptor.margin_top);
    window.set_margin(Edge::Bottom, descriptor.margin_bottom);

    // ... or like this
    // Anchors are if the window is pinned to each edge of the output
    let anchors = [
        (Edge::Left, descriptor.anchor_left),
        (Edge::Right, descriptor.anchor_right),
        (Edge::Top, descriptor.anchor_top),
        (Edge::Bottom, descriptor.anchor_bottom),
    ];

    for (anchor, state) in anchors {
        window.set_anchor(anchor, state);
    }

    window.set_namespace(&descriptor.namespace);

    window
}

fn set_window_layer(xid: u64, conn: &RustConnection) -> Result<(), ReplyError> {
    let state_atom = conn.intern_atom(false, b"_NET_WM_STATE")?.reply()?.atom;
    let layer_atom = conn
        .intern_atom(false, b"_NET_WM_STATE_BELOW")?
        .reply()?
        .atom;
    let attribute = ChangeWindowAttributesAux::new().override_redirect(1);

    conn.change_window_attributes(xid as u32, &attribute)?;

    conn.change_property(
        PropMode::REPLACE,
        xid as u32,
        state_atom,
        AtomEnum::ATOM,
        32,
        1,
        &layer_atom.to_ne_bytes(),
    )?;

    let values = ConfigureWindowAux::default().stack_mode(StackMode::BELOW);
    conn.configure_window(xid as u32, &values)?;

    conn.flush()?;

    Ok(())
}

fn x11_window(app: &Application, _descriptor: &WindowDescriptor) -> ApplicationWindow {
    let window = gtk4::ApplicationWindow::new(app);
    window.present();
    let xid = window
        .native()
        .unwrap()
        .surface()
        .unwrap()
        .downcast_ref::<gdk4_x11::X11Surface>()
        .unwrap()
        .xid();

    let (conn, _) = x11rb::connect(None).unwrap();
    if let Err(e) = set_window_layer(xid, &conn) {
        println!("Failed to create window: {}", e);
    }
    println!("Create window, id: {}", xid);

    window
}

// doesn't present the window
pub fn create_window(
    backend: &DisplayBackend,
    app: &Application,
    descriptor: &WindowDescriptor,
) -> ApplicationWindow {
    match backend {
        DisplayBackend::Wayland => wayland_window(app, descriptor),
        DisplayBackend::X11 => x11_window(app, descriptor),
    }
}
