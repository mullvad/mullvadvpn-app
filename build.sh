#!/usr/bin/env bash

# This script is used to build, and sign a release artifact. See `README.md` for further
# instructions.
#
# Invoke the script with --dev-build in order to skip checks, cleaning and signing.

set -eu

################################################################################
# Verify and configure environment.
################################################################################

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"
RUSTC_VERSION=`rustc +stable --version`
PRODUCT_VERSION=$(node -p "require('./gui/package.json').version" | sed -Ee 's/\.0//g')
CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-"$SCRIPT_DIR/target"}

source env.sh ""

if [[ "${1:-""}" != "--dev-build" ]]; then
    if [[ $(git diff --shortstat 2> /dev/null | tail -n1) != "" ]]; then
        echo "Dirty working directory!"
        echo "You should only build releases in clean working directories in order to make it"
        echo "easier to reproduce the same build."
        exit 1
    fi

    if [[ ("$(uname -s)" == "Darwin") ]]; then
        echo "Configuring environment for signing of binaries"
        if [[ -z ${CSC_LINK-} ]]; then
            echo "The variable CSC_LINK is not set. It needs to point to a file containing the"
            echo "private key used for signing of binaries."
            exit 1
        fi
        if [[ -z ${CSC_KEY_PASSWORD-} ]]; then
            read -sp "CSC_KEY_PASSWORD = " CSC_KEY_PASSWORD
            echo ""
            export CSC_KEY_PASSWORD
        fi
        # MacOs: This needs to be set to 'true' to activate signing, even when CSC_LINK is set.
        export CSC_IDENTITY_AUTO_DISCOVERY=true
    else
        unset CSC_LINK CSC_KEY_PASSWORD
        export CSC_IDENTITY_AUTO_DISCOVERY=false
    fi
else
    echo "!! Development build. Not for general distribution !!"
    unset CSC_LINK CSC_KEY_PASSWORD
    export CSC_IDENTITY_AUTO_DISCOVERY=false
fi

if [[ "${1:-""}" == "--dev-build" || $(git describe) != "$PRODUCT_VERSION" ]]; then
    GIT_COMMIT=$(git rev-parse --short HEAD)
    PRODUCT_VERSION="$PRODUCT_VERSION-dev-$GIT_COMMIT"
    echo "Modifying product version to $PRODUCT_VERSION"
else
    echo "Removing old Rust build artifacts"
    cargo +stable clean
fi

echo "Building Mullvad VPN $PRODUCT_VERSION"
SEMVER_VERSION=$(echo $PRODUCT_VERSION | sed -Ee 's/($|-.*)/.0\1/g')

function restore_metadata_backups() {
    pushd "$SCRIPT_DIR"
    mv gui/package.json.bak gui/package.json || true
    mv gui/package-lock.json.bak gui/package-lock.json || true
    mv Cargo.lock.bak Cargo.lock || true
    mv mullvad-daemon/Cargo.toml.bak mullvad-daemon/Cargo.toml || true
    mv mullvad-cli/Cargo.toml.bak mullvad-cli/Cargo.toml || true
    mv mullvad-problem-report/Cargo.toml.bak mullvad-problem-report/Cargo.toml || true
    mv talpid-openvpn-plugin/Cargo.toml.bak talpid-openvpn-plugin/Cargo.toml || true
    mv dist-assets/windows/version.h.bak dist-assets/windows/version.h || true
    mv gui/electron-builder.yml.bak gui/electron-builder.yml || true
    popd
}
trap 'restore_metadata_backups' EXIT

if [[ "${1:-""}" == "--dev-build" ]]; then
    # Disable installer compression on *explicit* dev builds.
    # This does not disable compression on build server builds, since they
    # always run without --dev-buid.
    echo "Disabling compression of installer in this dev build"
    cp gui/electron-builder.yml gui/electron-builder.yml.bak
    echo "compression: store" >> gui/electron-builder.yml
fi

sed -i.bak \
    -Ee "s/\"version\": \"[^\"]+\",/\"version\": \"$SEMVER_VERSION\",/g" \
    gui/package.json
cp gui/package-lock.json gui/package-lock.json.bak

cp Cargo.lock Cargo.lock.bak
sed -i.bak \
    -Ee "s/^version = \"[^\"]+\"\$/version = \"$SEMVER_VERSION\"/g" \
    mullvad-daemon/Cargo.toml \
    mullvad-cli/Cargo.toml \
    mullvad-problem-report/Cargo.toml \
    talpid-openvpn-plugin/Cargo.toml

SEMVER_ARRAY=($(echo $SEMVER_VERSION | sed -Ee 's/[.-]+/ /g'))
SEMVER_MAJOR=${SEMVER_ARRAY[0]}
SEMVER_MINOR=${SEMVER_ARRAY[1]}
SEMVER_PATCH=${SEMVER_ARRAY[2]}

cp dist-assets/windows/version.h dist-assets/windows/version.h.bak

cat <<EOF > dist-assets/windows/version.h
#define MAJOR_VERSION $SEMVER_MAJOR
#define MINOR_VERSION $SEMVER_MINOR
#define PATCH_VERSION $SEMVER_PATCH
#define PRODUCT_VERSION "$PRODUCT_VERSION"
EOF

################################################################################
# Compile and link all binaries.
################################################################################

if [[ "$(uname -s)" == "MINGW"* ]]; then
    CPP_BUILD_MODES="Release" ./build_windows_modules.sh $@
fi

echo "Building Rust code in release mode using $RUSTC_VERSION..."
MULLVAD_ADD_MANIFEST="1" cargo +stable build --release

################################################################################
# Other work to prepare the release.
################################################################################

if [[ ("$(uname -s)" == "Darwin") ]]; then
    binaries=(
        mullvad-daemon
        mullvad
        problem-report
        libtalpid_openvpn_plugin.dylib
    )
elif [[ ("$(uname -s)" == "Linux") ]]; then
    binaries=(
        mullvad-daemon
        mullvad
        problem-report
        libtalpid_openvpn_plugin.so
    )
elif [[ ("$(uname -s)" == "MINGW"*) ]]; then
    binaries=(
        mullvad-daemon.exe
        mullvad.exe
        problem-report.exe
        talpid_openvpn_plugin.dll
    )
fi
for binary in ${binaries[*]}; do
    SRC="$CARGO_TARGET_DIR/release/$binary"
    DST="$SCRIPT_DIR/dist-assets/$binary"
    if [[ "$(uname -s)" == "MINGW"* || "$binary" == *.dylib ]]; then
        echo "Copying $SRC => $DST"
        cp "$SRC" "$DST"
    else
        echo "Stripping $SRC => $DST"
        strip "$SRC" -o "$DST"
    fi
done


echo "Updating relay list..."
set +e
read -d '' JSONRPC_CODE <<-JSONRPC_CODE
var buff = "";
process.stdin.on('data', function (chunk) {
    buff += chunk;
})
process.stdin.on('end', function () {
    var obj = JSON.parse(buff);
    var output = JSON.stringify(obj.result, null, '    ');
    process.stdout.write(output);
})
JSONRPC_CODE
set -e

JSONRPC_RESPONSE="$(curl -X POST \
    --fail \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc": "2.0", "id": "0", "method": "relay_list_v2"}' \
     https://api.mullvad.net/rpc/)"
echo $JSONRPC_RESPONSE | node -e "$JSONRPC_CODE" >  dist-assets/relays.json


pushd "$SCRIPT_DIR/gui"

echo "Installing JavaScript dependencies..."
npm install

################################################################################
# Package release.
################################################################################

echo "Packing final release artifact..."
case "$(uname -s)" in
    Linux*)     npm run pack:linux;;
    Darwin*)    npm run pack:mac;;
    MINGW*)     npm run pack:win;;
esac

popd

for semver_path in dist/*$SEMVER_VERSION*; do
    product_path=$(echo $semver_path | sed -Ee "s/$SEMVER_VERSION/$PRODUCT_VERSION/g")
    echo "Moving $semver_path -> $product_path"
    mv $semver_path $product_path
done

echo "**********************************"
echo ""
echo " The build finished successfully! "
echo " You have built:"
echo ""
echo " $PRODUCT_VERSION"
echo ""
echo "**********************************"
