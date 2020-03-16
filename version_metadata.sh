#!/usr/bin/env bash
#
# Can inject correctly formatted version strings/numbers in all the various
# project metadata files. Can also back them up and restore them.

set -eu

# Regex that only matches valid Mullvad VPN versions. It also captures
# relevant values into capture groups, read out via BASH_REMATCH[x].
VERSION_REGEX="^20([0-9]{2})\.([1-9][0-9]?)(-beta([1-9][0-9]?))?(-dev-[0-9a-f]+)?$"

case "$1" in
    "inject")
        PRODUCT_VERSION=$2
        if [[ ! $PRODUCT_VERSION =~ $VERSION_REGEX ]]; then
            echo "Invalid version format. Please specify version as:"
            echo "<YEAR>.<NUMBER>[-beta<NUMBER>]"
            exit 1
        fi

        VERSION_YEAR=$(printf "%02d" ${BASH_REMATCH[1]})
        VERSION_NUMBER=$(printf "%02d" ${BASH_REMATCH[2]})
        VERSION_PATCH="00" # Not used for now.
        VERSION_BETA=$(printf "%02d" ${BASH_REMATCH[4]:-99})
        ANDROID_VERSION_CODE=${VERSION_YEAR}${VERSION_NUMBER}${VERSION_PATCH}${VERSION_BETA}

        SEMVER_VERSION=$(echo $PRODUCT_VERSION | sed -Ee 's/($|-.*)/.0\1/g')
        SEMVER_MAJOR="20${BASH_REMATCH[1]}"
        SEMVER_MINOR=${BASH_REMATCH[2]}
        SEMVER_PATCH="0"

        # Electron GUI
        cp gui/package.json gui/package.json.bak
        cp gui/package-lock.json gui/package-lock.json.bak
        (cd gui/ && npm version $SEMVER_VERSION --no-git-tag-version --allow-same-version)

        # Rust crates
        sed -i.bak -Ee "s/^version = \"[^\"]+\"\$/version = \"$SEMVER_VERSION\"/g" \
            mullvad-daemon/Cargo.toml \
            mullvad-cli/Cargo.toml \
            mullvad-problem-report/Cargo.toml \
            mullvad-setup/Cargo.toml \
            talpid-openvpn-plugin/Cargo.toml

        # Windows C++
        cp dist-assets/windows/version.h dist-assets/windows/version.h.bak
        cat <<EOF > dist-assets/windows/version.h
#define MAJOR_VERSION $SEMVER_MAJOR
#define MINOR_VERSION $SEMVER_MINOR
#define PATCH_VERSION $SEMVER_PATCH
#define PRODUCT_VERSION "$PRODUCT_VERSION"
EOF

        # Android
        if [[ ("$(uname -s)" == "Linux") ]]; then
            cp android/build.gradle android/build.gradle.bak
            sed -i -Ee "s/versionCode [0-9]+/versionCode $ANDROID_VERSION_CODE/g" \
                android/build.gradle
            sed -i -Ee "s/versionName \"[^\"]+\"/versionName \"$PRODUCT_VERSION\"/g" \
                android/build.gradle
        fi
        ;;
    "restore-backup")
        # Electron GUI
        mv gui/package.json.bak gui/package.json || true
        mv gui/package-lock.json.bak gui/package-lock.json || true
        # Rust crates
        mv mullvad-daemon/Cargo.toml.bak mullvad-daemon/Cargo.toml || true
        mv mullvad-cli/Cargo.toml.bak mullvad-cli/Cargo.toml || true
        mv mullvad-problem-report/Cargo.toml.bak mullvad-problem-report/Cargo.toml || true
        mv mullvad-setup/Cargo.toml.bak mullvad-setup/Cargo.toml || true
        mv talpid-openvpn-plugin/Cargo.toml.bak talpid-openvpn-plugin/Cargo.toml || true
        # Windows C++
        mv dist-assets/windows/version.h.bak dist-assets/windows/version.h || true
        # Android
        if [[ ("$(uname -s)" == "Linux") ]]; then
            mv android/build.gradle.bak android/build.gradle || true
        fi
        ;;
    "delete-backup")
        # Electron GUI
        rm gui/package.json.bak || true
        rm gui/package-lock.json.bak || true
        # Rust crates
        rm mullvad-daemon/Cargo.toml.bak || true
        rm mullvad-cli/Cargo.toml.bak || true
        rm mullvad-problem-report/Cargo.toml.bak || true
        rm mullvad-setup/Cargo.toml.bak || true
        rm talpid-openvpn-plugin/Cargo.toml.bak || true
        # Windows C++
        rm dist-assets/windows/version.h.bak || true
        # Android
        if [[ ("$(uname -s)" == "Linux") ]]; then
            rm android/build.gradle.bak || true
        fi
        ;;
    *)
        echo "Invalid command"
        exit 1
        ;;
esac
