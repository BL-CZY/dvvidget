use glib::object::Cast;
use gtk4::prelude::{GtkWindowExt, NativeExt, WidgetExt};
use gtk4::{Application, ApplicationWindow};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use serde::Deserialize;
use serde_inline_default::serde_inline_default;
use smart_default::SmartDefault;
use toml::map::Map;
use toml::value::Value;

use x11rb::connection::Connection;
use x11rb::errors::ReplyError;
use x11rb::protocol::xproto::{
    AtomEnum, ChangeWindowAttributesAux, ConnectionExt, EventMask, PropMode, StackMode,
};
use x11rb::rust_connection::RustConnection;

use crate::utils::DisplayBackend;

#[serde_inline_default]
#[derive(Clone, Deserialize, SmartDefault, Debug)]
pub struct WindowDescriptor {
    #[serde(deserialize_with = "deserialize_layer")]
    #[default(_code = "Layer::Overlay")]
    #[serde_inline_default(Layer::Overlay)]
    pub layer: Layer,

    #[serde_inline_default(0)]
    pub margin_left: i32,
    #[serde_inline_default(0)]
    pub margin_right: i32,
    #[serde_inline_default(0)]
    pub margin_top: i32,
    #[serde_inline_default(0)]
    pub margin_bottom: i32,

    #[serde_inline_default(false)]
    pub anchor_left: bool,
    #[serde_inline_default(false)]
    pub anchor_right: bool,
    #[serde_inline_default(false)]
    pub anchor_top: bool,
    #[serde_inline_default(false)]
    pub anchor_bottom: bool,

    #[serde_inline_default(false)]
    pub exclusive: bool,

    #[serde(skip_deserializing)]
    pub keyboard_mode: KeyboardModeWrapper,

    #[serde_inline_default(true)]
    pub visible_on_start: bool,
    #[serde_inline_default("dvvidget".into())]
    pub namespace: String,
}

#[derive(Clone, Debug)]
pub struct KeyboardModeWrapper {
    pub inner: KeyboardMode,
}

impl Default for KeyboardModeWrapper {
    fn default() -> Self {
        Self {
            inner: KeyboardMode::None,
        }
    }
}

fn deserialize_layer<'de, D>(deserializer: D) -> Result<Layer, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(string_to_layer(&s))
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

fn wayland_window(
    app: &Application,
    descriptor: &WindowDescriptor,
    mode: KeyboardMode,
    monitor: &gtk4::gdk::Monitor,
) -> ApplicationWindow {
    // Create a normal GTK window however you like
    let window = gtk4::ApplicationWindow::new(app);

    // Before the window is first realized, set it up to be a layer surface
    window.init_layer_shell();

    // Display above normal windows
    window.set_layer(descriptor.layer);

    window.set_keyboard_mode(mode);

    window.set_monitor(Some(monitor));

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

    window.set_namespace(Some(&descriptor.namespace));

    window
}

fn set_window_layer(xid: u32, conn: &RustConnection) -> Result<(), ReplyError> {
    conn.unmap_window(xid)?;

    let override_attr = ChangeWindowAttributesAux::new()
        .override_redirect(1)
        .event_mask(EventMask::SUBSTRUCTURE_REDIRECT);
    conn.change_window_attributes(xid as u32, &override_attr)?;

    conn.flush()?;

    let wm_window_type_atom = conn
        .intern_atom(false, b"_NET_WM_WINDOW_TYPE")?
        .reply()?
        .atom;
    let wm_window_type_notification_atom = conn
        .intern_atom(false, b"_NET_WM_WINDOW_TYPE_NOTIFICATION")?
        .reply()?
        .atom;
    let wm_state_atom = conn.intern_atom(false, b"_NET_WM_STATE")?.reply()?.atom;
    let wm_state_sticky_atom = conn
        .intern_atom(false, b"_NET_WM_STATE_STICKY")?
        .reply()?
        .atom;
    let wm_state_skip_taskbar_atom = conn
        .intern_atom(false, b"_NET_WM_STATE_SKIP_TASKBAR")?
        .reply()?
        .atom;
    let wm_state_skip_pager_atom = conn
        .intern_atom(false, b"_NET_WM_STATE_SKIP_PAGER")?
        .reply()?
        .atom;

    // Set _NET_WM_WINDOW_TYPE property
    conn.change_property(
        PropMode::REPLACE,
        xid,
        wm_window_type_atom,
        AtomEnum::ATOM,
        32, // 32 bits per element
        1,  // 1 element
        &wm_window_type_notification_atom.to_ne_bytes(),
    )?;

    // _NET_WM_STATE
    conn.change_property(
        PropMode::REPLACE,
        xid,
        wm_state_atom,
        AtomEnum::ATOM,
        32, // 32 bits per element
        1,
        &wm_state_sticky_atom.to_ne_bytes(),
    )?;

    conn.change_property(
        PropMode::REPLACE,
        xid,
        wm_state_atom,
        AtomEnum::ATOM,
        32, // 32 bits per element
        1,
        &wm_state_skip_taskbar_atom.to_ne_bytes(),
    )?;

    conn.change_property(
        PropMode::REPLACE,
        xid,
        wm_state_atom,
        AtomEnum::ATOM,
        32, // 32 bits per element
        1,
        &wm_state_skip_pager_atom.to_ne_bytes(),
    )?;

    conn.flush()?;

    // After window creation but before mapping it
    let values = x11rb::protocol::xproto::ConfigureWindowAux::new()
        .x(200) // x position
        .y(300) // y position
        .width(400) // width
        .height(300) // height
        .stack_mode(StackMode::BELOW);

    conn.configure_window(xid, &values)?;
    conn.flush()?;

    conn.map_window(xid)?;
    conn.flush()?;

    Ok(())
}

fn x11_window(app: &Application, _descriptor: &WindowDescriptor) -> ApplicationWindow {
    let (conn, _) = x11rb::connect(None).unwrap();
    let window = gtk4::ApplicationWindow::new(app);
    window.present();

    let xid = window
        .native()
        .unwrap()
        .surface()
        .unwrap()
        .downcast_ref::<gdk4_x11::X11Surface>()
        .unwrap()
        .xid() as u32;

    if let Err(e) = set_window_layer(xid, &conn) {
        println!("Failed to create window: {}", e);
    }
    println!("Create window, id: {}", xid);

    window.set_visible(true);

    window
}

// doesn't present the window
pub fn create_window(
    backend: &DisplayBackend,
    app: &Application,
    descriptor: &WindowDescriptor,
    mode: KeyboardMode,
    monitor: &gtk4::gdk::Monitor,
) -> ApplicationWindow {
    match backend {
        DisplayBackend::Wayland => wayland_window(app, descriptor, mode, monitor),
        DisplayBackend::X11 => x11_window(app, descriptor),
    }
}
