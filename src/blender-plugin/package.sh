#!/bin/bash

set -e

PLUGIN_DIR_NAME="text_to_face_blender"
# Extract version from ../../Cargo.toml
VERSION=$(grep '^version' ../../Cargo.toml | head -n1 | cut -d '"' -f2)
OUTPUT_ZIP="text_to_face_blender-v$VERSION.zip"
TMP_DIR="/tmp/text_to_face_blender_pack_$$"

# Clean up temp dir on exit
trap "rm -rf \"$TMP_DIR\"" EXIT

# Build the Rust binary (release mode)
( cd ../.. && cargo build --release )

# Copy the built binary into the plugin directory
cp ../../target/release/text-to-face ./

# Create temp directory structure
mkdir -p "$TMP_DIR/$PLUGIN_DIR_NAME"

# Copy all plugin files except the script and zip itself, and exclude __pycache__ and .DS_Store
rsync -av --exclude="package.sh" --exclude="text_to_face_blender-*.zip" --exclude="__pycache__" --exclude="*.DS_Store" ./ "$TMP_DIR/$PLUGIN_DIR_NAME/"

# Create the zip with the top-level directory only
cd "$TMP_DIR"
zip -r "$OLDPWD/$OUTPUT_ZIP" "$PLUGIN_DIR_NAME"
cd "$OLDPWD"

# Remove the copied binary from the plugin directory after packaging
rm -f ./text-to-face

echo "Packaged Blender plugin as $OUTPUT_ZIP with top-level $PLUGIN_DIR_NAME directory and bundled binary" 