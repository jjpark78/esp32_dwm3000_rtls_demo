#!/bin/zsh
cargo build --features experimental
sshpass -p qkrwowls9794\!A ssh jaejinpark@192.168.0.250 'uwb-anchor-flash.bat'
