#!/bin/bash

# Check if the script is run as root
if [ "$EUID" -ne 0 ]; then
    echo "ERROR: Please run as root or use sudo."
    exit 1
fi

# URL of the file to download
FILE_URL="https://raw.githubusercontent.com/skubed0007/gob/main/bin/gob_gcc"

# Destination directory
DEST_DIR="/usr/local/bin"

echo "===================================================="
echo "Starting Gob installation..."
echo "===================================================="
echo ""
echo "Step 1: Verifying destination directory..."
echo "Destination: $DEST_DIR"
if [ ! -d "$DEST_DIR" ]; then
    echo "Destination directory does not exist. Creating it..."
    mkdir -p "$DEST_DIR" || { echo "ERROR: Failed to create destination directory."; exit 1; }
fi
echo "Destination directory exists."
echo ""

echo "Step 2: Downloading file from:"
echo "$FILE_URL"
DOWNLOAD_PATH="$DEST_DIR/$(basename "$FILE_URL")"
curl -o "$DOWNLOAD_PATH" "$FILE_URL" && echo "Download completed successfully." || { echo "ERROR: File download failed."; exit 1; }
echo ""

echo "Step 3: Renaming downloaded file to 'gob'..."
mv "$DOWNLOAD_PATH" "$DEST_DIR/gob" && echo "File renamed to 'gob' successfully." || { echo "ERROR: File renaming failed."; exit 1; }
echo ""

echo "Step 4: Setting executable permissions on 'gob'..."
chmod +x "$DEST_DIR/gob" && echo "Executable permissions set successfully." || { echo "ERROR: Failed to set executable permissions."; exit 1; }
echo ""

echo "===================================================="
echo "Gob installation is complete!"
echo "----------------------------------------------------"
echo "You can now run 'gob' from the terminal."
echo "Please run \"gob update\" to fetch the package index and setup directories on your PATH."
echo "===================================================="
