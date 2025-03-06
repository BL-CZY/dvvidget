# Dvvidget 
Hi! This the widget system that I wrote for my own desktop <br />
Here is a list of compositors on which it works

-- Hyprland: ✓ <br />
-- Sway: ✓ <br />

### How to build
You'll need ```gtk4-layer-shell, wpctl, Rust, and brightnessctl``` as a dependency, 
and simply running ```./install.sh``` would do the job.

### Usage
Use ```-h/--help``` to print help <br />
Dvvidget supports custom configs and css files,
you can use ```-c``` or ```--config``` flag to set the config. The default path is ```$HOME/.config/dvvidget/config.toml```
you can use ```-p``` or ```--path``` flag to set the socket path. The default path is ```/tmp/dvvidget-[version].sock```
<br />
Here is a sample config file:
```toml
[general]
css_path = "/path/to/style.css"

[vol]
enable = true
max_vol = 100
use_svg = true
icons = [
    { "lower" = 0, "upper" = 5, "icon" = "/path/to/vol-off.svg" }, # if use_svg is set, put the path of the icon here, otherwise, put the glyph
    { "lower" = 5, "upper" = 35, "icon" = "/path/to/vol-low.svg" },
    { "lower" = 35, "upper" = 70, "icon" = "/path/to/vol-mid.svg" },
    { "lower" = 70, "upper" = 101, "icon" = "/path/to/vol-high.svg" },
]
mute_icon = "/path/to/vol-mute.svg"

[vol.window]
visible_on_start = false
anchor_bottom = true
margin_bottom = 130

[bri]
enable = true
use_svg = true
icons = [
    { "lower" = 0, "upper" = 50, "icon"= "/path/to/bri-low.svg" },
    { "lower" = 50, "upper" = 101, "icon"= "/path/to/bri-high.svg" },
]

[bri.window]
visible_on_start = false
anchor_bottom = true
margin_bottom = 130

[dvoty]
enable = true
max_height = 300
instruction_icon = "/path/to/dvoty/instruction.svg"
math_icon = "/path/to/dvoty/math.svg"
search_icon = "/path/to/dvoty/search.svg"
cmd_icon = "/path/to/dvoty/cmd.svg"
url_icon = "/path/to/dvoty/url.svg"
letter_icon = "/path/to/dvoty/letter.svg"
launch_icon = "/path/to/dvoty/app.svg"
search_engine = "google" # other options are ddg and wikipedia
terminal_exec = "kitty" # this terminal will be used to run commands. For example, if you want to use Alacritty, set this to Alacritty -e
spacing = 0

[dvoty.window]
visible_on_start = false
layer = "top"
```
There is a style.css in src that has a sample css. <br />
It uses gtk css. <br />

Here are the class names:<br />

for sound and brightness: <br />
|--window: .sound-window & .bri-window <br />
&nbsp;&nbsp;&nbsp;&nbsp;|--box: <br />
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;|--icon: .sound-icon & .bri-icon <br />
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;|--label: .sound-label & .bri-label <br />
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;|--scale: .sound-scale & .bri-scale <br />

for the launcher (dvoty): <br />
|--window: .dvoty-window: <br />
&nbsp;&nbsp;&nbsp;&nbsp;|--wrapper: .dvoty-wrapper <br />
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;|--input: .dvoty-input <br />
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;|--outerwrapper: .dvoty-scroll <br />
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;|--scrolled window <br />
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;|--list box: .dvoty-list <br />
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;|-- ENTRY

for ENTRY: <br />
|--list box row:  
*Every entry will have the class .dvoty-entry if it's not focused and .dvoty-entry-select if it's focused*<br/>
*Aside from that, every entry will get .dvoty-entry-[type] or .dvoty-entry-[type]-select* <br />
*Types are: math, instruction, search, url, launch, and letter* <br />
&nbsp;&nbsp;&nbsp;&nbsp;|--box: .dvoty-box<br />
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;|--icon: .dvoty-icon<br />
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;|--label: .dvoty-label, .dvoty-label-mid<br />
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;|--label: .dvoty-label, .dvoty-label-end<br />
