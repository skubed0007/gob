#!/bin/bash

# Check if the script is run as root
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root or use sudo"
    exit 1
fi

# URL of the file to download
FILE_URL=""

# Destination directory
DEST_DIR="/usr/local/bin"

# Download the file
curl -o "$DEST_DIR/$(basename $FILE_URL)" "$FILE_URL"

# Make the file executable
chmod +x "$DEST_DIR/$(basename $FILE_URL)"

echo "File downloaded and placed in $DEST_DIR, and made executable."