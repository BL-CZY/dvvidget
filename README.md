# Dvvidget 
Hi! This the widget system that I wrote for my own desktop <br />
Here is a list of compositors on which it works

-- Hyprland: ✓ <br />
-- Sway: unknown <br />

### How to build
You'll need ```gtk4-layer-shell``` as a dependency, 
and simply running ```./install.sh``` would do the job.

In order to get the brightness module to work, please set up your backlight as stated in the [Arch Wiki](https://wiki.archlinux.org/title/Backlight)

### Usage
Use ```-h/--help``` to print help <br />
Dvvidget supports custom configs and css files,
you can use ```-c``` or ```--config``` flag to set the path. The default path is ```$HOME/.config/dvvidget/config.toml```
<br />
Here is a sample config file:
```toml
[general]
css_path = "absolute/path/to/your/css"

[volume]
max_vol = 100
run_cmd = "wpctl"

[volume.window]
visible_on_start = false
exclusive = false
anchor_left = false
anchor_right = false
anchor_top = false
anchor_bottom = true
margin_left = 0
margin_right = 0
margin_top = 0
margin_bottom = 130
```
There is a style.css in src that has a sample css
It uses gtk css
