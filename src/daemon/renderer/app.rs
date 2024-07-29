use gio::ApplicationFlags;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};

fn activate(application: &gtk4::Application) {
    // Create a normal GTK window however you like
    let window = gtk4::ApplicationWindow::new(application);

    // Before the window is first realized, set it up to be a layer surface
    window.init_layer_shell();

    // Display above normal windows
    window.set_layer(Layer::Overlay);

    // Push other windows out of the way
    // window.auto_exclusive_zone_enable();

    // The margins are the gaps around the window's edges
    // Margins and anchors can be set like this...
    window.set_margin(Edge::Left, 40);
    window.set_margin(Edge::Right, 40);
    window.set_margin(Edge::Top, 20);

    // ... or like this
    // Anchors are if the window is pinned to each edge of the output
    let anchors = [
        (Edge::Left, true),
        (Edge::Right, true),
        (Edge::Top, false),
        (Edge::Bottom, true),
    ];

    for (anchor, state) in anchors {
        window.set_anchor(anchor, state);
    }

    // Set up a widget
    let label = gtk4::Label::new(Some(""));
    label.set_markup("<span font_desc=\"20.0\">GTK Layer Shell example!</span>");
    window.set_child(Some(&label));
    window.show()
}

pub fn start_app() {
    let app = gtk4::Application::new(Some("org.dvida.dvvidgets"), ApplicationFlags::default());

    app.connect_activate(|app| activate(&app));

    app.run_with_args(&[""]);
}
