#!/bin/sh
# Cut the mouse picture from the official Combaterwing GM software.
# Usage: ./install-picture.sh "<path to Combaterwing GM folder>"
set -e
mkdir -p "$HOME/.local/share/luom-mouse"
magick "$1/Skins/Std/BasicSet2.png" -crop 245x300+0+185 +repage "$HOME/.local/share/luom-mouse/mouse.png"
