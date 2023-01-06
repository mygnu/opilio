#!/bin/bash

declare -a WORKSPACES=(opilio-daemon opilio-gui)

for i in "${WORKSPACES[@]}"
do : 
   (cd "$i" && cargo build --release)
done

(cd "opilio-tui" && cargo deb)
