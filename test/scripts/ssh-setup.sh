#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd $SCRIPT_DIR

RUNNER_DIR="$1"
CURRENT_APP="$2"
PREVIOUS_APP="$3"
UI_RUNNER="$4"

# Copy over test runner to correct place

echo "Copying test-runner to $RUNNER_DIR"

mkdir -p $RUNNER_DIR

for file in test-runner $CURRENT_APP $PREVIOUS_APP $UI_RUNNER openvpn.ca.crt; do
    echo "Moving $file to $RUNNER_DIR"
    cp -f "$SCRIPT_DIR/$file" $RUNNER_DIR
done

chown -R root "$RUNNER_DIR/"

# Create service

function setup_macos {
    RUNNER_PLIST_PATH="/Library/LaunchDaemons/net.mullvad.testunner.plist"

    echo "Creating test runner service as $RUNNER_PLIST_PATH"

    cat > $RUNNER_PLIST_PATH << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>net.mullvad.testrunner</string>

    <key>ProgramArguments</key>
    <array>
        <string>$RUNNER_DIR/test-runner</string>
        <string>/dev/tty.virtio</string>
        <string>serve</string>
    </array>

    <key>UserName</key>
    <string>root</string>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <true/>

    <key>StandardOutPath</key>
    <string>/tmp/runner.out</string>

    <key>StandardErrorPath</key>
    <string>/tmp/runner.err</string>

    <key>EnvironmentVariables</key>
    <dict>
        <key>PATH</key>
        <string>/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:/usr/local/sbin</string>
    </dict>
</dict>
</plist>
EOF

    echo "Starting test runner service"

    launchctl load -w $RUNNER_PLIST_PATH
}

function setup_systemd {
    RUNNER_SERVICE_PATH="/etc/systemd/system/testrunner.service"

    echo "Creating test runner service as $RUNNER_SERVICE_PATH"

    cat > $RUNNER_SERVICE_PATH << EOF
[Unit]
Description=Mullvad Test Runner

[Service]
ExecStart=$RUNNER_DIR/test-runner /dev/ttyS0 serve

[Install]
WantedBy=multi-user.target
EOF

    echo "Starting test runner service"

    semanage fcontext -a -t bin_t "$RUNNER_DIR/.*" &> /dev/null || true

    systemctl enable testrunner.service
    systemctl start testrunner.service
}

if [[ "$(uname -s)" == "Darwin" ]]; then
    setup_macos
    exit 0
fi

setup_systemd

function install_packages_apt {
    apt update
    apt install -yf xvfb wireguard-tools curl
    curl -fsSL https://get.docker.com | sh
}

# Install required packages
if which apt &>/dev/null; then
    install_packages_apt
elif which dnf &>/dev/null; then
    dnf install -y xorg-x11-server-Xvfb wireguard-tools podman
fi
