#! /usr/bin/env bash
set -eu

WD=$(pwd)/$(dirname $0)
GIT_DIR=libzmq-git
INSTALL_DIR=linux

### Install build deps ###
sudo apt-get update
sudo apt-get -y install git build-essential libtool autoconf automake uuid-dev pkg-config

### Get the code ###
if [ -e "$WD/$GIT_DIR/.git" ]; then
    cd "$WD/$GIT_DIR"
    git fetch
    git checkout origin/master
    cd -
else
    git clone --depth=1 git@github.com:zeromq/zeromq4-1.git "$WD/$GIT_DIR"
fi

### Build ###
## We skip running the tests here as we trust the zmq maintainers to publish
## working code :) Living life on the wild side
trap "cd -" EXIT
cd "$WD/$GIT_DIR"

set -x
./autogen.sh
./configure --prefix="$WD/$INSTALL_DIR"    # add other options here
make
sudo make install
sudo ldconfig
set +x

echo "WOOWZ, it's built now. All the good stuff is in $WD/$INSTALL_DIR"
