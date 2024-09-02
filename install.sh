#!/bin/bash

cargo build --release
sudo cp target/release/dvvidget /usr/bin/dvvidget
sudo mkdir -p /usr/share/dvvidget/
sudo cp src/style.css /usr/share/dvvidget/style.css
