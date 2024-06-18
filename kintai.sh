#!/bin/bash

# Required parameters:
# @raycast.schemaVersion 1
# @raycast.title kintai
# @raycast.mode silent
# @raycast.refreshTime 1h
# @raycast.argument1 { "type": "text", "placeholder": "command", "percentEncoded": false }
# @raycast.argument2 { "type": "text", "placeholder": "file", "percentEncoded": false }

# Optional parameters:
# @raycast.icon 🤖

# Documentation:
# @raycast.author yochidros
# @raycast.authorURL https://raycast.com/yochidros

file_path=$2
command=$1
~/.cargo/bin/kintai -f $file_path $command
