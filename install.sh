#!/bin/bash

# Determine the original non-root user
if [ "$SUDO_USER" ]; then
    USER_NAME=$SUDO_USER
else
    USER_NAME=$(whoami)
fi

# Determine the home directory of the user
USER_HOME=$(eval echo ~$USER_NAME)

echo -e "Installing Steam Patch...\n"
cd "$USER_HOME"

# Remove any existing steam-patch directory or file
sudo rm -rf "$USER_HOME/steam-patch/"
sudo rm -f /usr/bin/steam-patch

# Clone the repository to get auxiliary files like .service files
git clone https://github.com/corando98/steam-patch "$USER_HOME/steam-patch"
cd "$USER_HOME/steam-patch"
CURRENT_WD=$(pwd)

# Enable CEF debugging
touch "$USER_HOME/.steam/steam/.cef-enable-remote-debugging"

# Download the latest steam-patch binary from GitHub releases
curl -L $(curl -s https://api.github.com/repos/corando98/steam-patch/releases/latest | grep "browser_download_url" | cut -d '"' -f 4) -o steam-patch

# Make the binary executable
chmod +x steam-patch

# Move the binary to a system path
sudo mv steam-patch /usr/bin/

# Replace placeholder with actual username in steam-patch.service
sed -i "s/@USER@/$USER_NAME/g" steam-patch.service

# Copy service files and enable services
sudo cp steam-patch.service /etc/systemd/system/
sudo cp restart-steam-patch-on-boot.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl stop handycon
sudo systemctl disable handycon
sudo systemctl enable steam-patch.service
sudo systemctl start steam-patch.service
sudo systemctl enable restart-steam-patch-on-boot.service
sudo systemctl start restart-steam-patch-on-boot.service

# Handle steamos-polkit-helpers update
STEAMOS_POLKIT_DIR="/usr/bin/steamos-polkit-helpers"
if [ -f "$STEAMOS_POLKIT_DIR/steamos-priv-write" ]; then
    sudo cp "$STEAMOS_POLKIT_DIR/steamos-priv-write" "$STEAMOS_POLKIT_DIR/steamos-priv-write-bkp"
fi
sudo cp steamos-priv-write-updated "$STEAMOS_POLKIT_DIR/steamos-priv-write"

# Gracefully shut down Steam (replace with the specific command if different)
steamrestart

# Wait for a moment to ensure Steam has completely shut down
sleep 2

echo -e "\nSteam Patch installation completed."
