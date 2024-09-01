use gtk4::{Application, ApplicationWindow};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use toml::map::Map;
use toml::value::Value;

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

    pub visible_on_start: bool,
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
    pub fn new() -> Self {
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

            visible_on_start: true,
        }
    }

    pub fn from_val(
        layer: Layer,
        margin_left: i32,
        margin_right: i32,
        margin_top: i32,
        margin_bottom: i32,
        anchor_left: bool,
        anchor_right: bool,
        anchor_top: bool,
        anchor_bottom: bool,
        exclusive: bool,
        visible_on_start: bool,
    ) -> Self {
        WindowDescriptor {
            layer,
            margin_left,
            margin_right,
            margin_top,
            margin_bottom,
            anchor_left,
            anchor_right,
            anchor_top,
            anchor_bottom,
            exclusive,
            visible_on_start,
        }
    }

    pub fn vol_from_toml(toml: &Map<String, Value>) -> WindowDescriptor {
        let mut result = WindowDescriptor::new();
        result.anchor_bottom = true;
        result.margin_bottom = 130;

        let inner = if let Some(outer) = toml.get("volume") {
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

// doesn't present the window
pub fn create_window(app: &Application, descriptor: WindowDescriptor) -> ApplicationWindow {
    // Create a normal GTK window however you like
    let window = gtk4::ApplicationWindow::new(app);

    // Before the window is first realized, set it up to be a layer surface
    window.init_layer_shell();

    // Display above normal windows
    window.set_layer(descriptor.layer);

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

    window
}
