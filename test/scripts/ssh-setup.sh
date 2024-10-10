#!/usr/bin/env bash

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

RUNNER_DIR="$1"
APP_PACKAGE="$2"
PREVIOUS_APP="$3"
UI_RUNNER="$4"
UNPRIVILEGED_USER="$5"

# Copy over test runner to correct place

echo "Copying test-runner to $RUNNER_DIR"

mkdir -p "$RUNNER_DIR"

for file in test-runner connection-checker $APP_PACKAGE $PREVIOUS_APP $UI_RUNNER; do
    echo "Moving $SCRIPT_DIR/$file to $RUNNER_DIR"
    cp -f "$SCRIPT_DIR/$file" "$RUNNER_DIR"
done

# Unprivileged users need execute rights for some executables
chmod 775 "${RUNNER_DIR}/connection-checker"
chmod 775 "${RUNNER_DIR}/$UI_RUNNER"

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

    create_test_user_macos

    echo "Starting test runner service"

    launchctl load -w $RUNNER_PLIST_PATH
}

function create_test_user_macos {
    echo "Adding test user account"
    sysadminctl -addUser "$UNPRIVILEGED_USER" -fullName "$UNPRIVILEGED_USER" -password "$UNPRIVILEGED_USER"
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

    create_test_user_linux

    systemctl enable testrunner.service
    systemctl start testrunner.service
}

function create_test_user_linux {
    echo "Adding test user account"
    useradd -m "$UNPRIVILEGED_USER"
    echo "$UNPRIVILEGED_USER:$UNPRIVILEGED_USER" | chpasswd
}

if [[ "$(uname -s)" == "Darwin" ]]; then
    setup_macos
    exit 0
fi

setup_systemd

# Run apt with some arguments
robust_apt () {
    # We don't want to fail due to the global apt lock being
    # held, which happens sporadically. It is fine to wait for
    # some time if it means that the test run can continue.
    DEBIAN_FRONTEND=noninteractive apt-get -qy -o DPkg::Lock::Timeout=60 "$@"
}

function install_packages_apt {
    echo "Installing required apt packages"
    robust_apt update
    robust_apt install xvfb wireguard-tools curl
    if ! which ping &>/dev/null; then
        robust_apt install iputils-ping
    fi
    curl -fsSL https://get.docker.com | sh
}

# Install required packages
if which apt &>/dev/null; then
    install_packages_apt
elif which dnf &>/dev/null; then
    dnf install -y xorg-x11-server-Xvfb wireguard-tools podman
fi
