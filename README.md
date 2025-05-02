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
you can use ```-m``` or ```--monitor``` flag to run commands for a specific monitor
<br />
The sample config file is in src/config.toml, and you can put the config in ~/.config/dvvidget/config.toml <br />

dvvidget has a client and a server. If you want to use dvvidget, you can use ```dvvidget daemon``` to start the daemon. 
You can then use ```dvvidget volume -h```, ```dvvidget brightness -h```, and ```dvvidget dvoty -h``` to learn how to 
use the client. <br />

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
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;|--scrolled window: .dvoty-scroll-mid<br />
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;|--label: .dvoty-label .dvoty-label-mid<br />
&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;|--label: .dvoty-label, .dvoty-label-end<br />

Dependencies: <br />
wpctl, brightnessctl, gvfs, gtk4-layer-shell
