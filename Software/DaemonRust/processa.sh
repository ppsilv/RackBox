#!/bin/bash

cargo build --release
sudo systemctl stop rackbox-led
cp ./target/release/led_daemon ~/bin/.
sudo systemctl restart rackbox-led

