#! /usr/bin/env bash
set -eu

WD=$(pwd)/$(dirname $0)
GIT_DIR=libzmq-git
INSTALL_DIR=$(readlink -f $1)
USE_CACHE=${USE_CACHE:-"1"}




### TEST IF ZMQ IS BUILT ###
F1=$INSTALL_DIR/include/zmq.h
F2=$INSTALL_DIR/lib/libzmq.so
if [[ -f $F1 && -f $F2 ]]; then
    ZMQ_IS_BUILT=1
else
    ZMQ_IS_BUILT=0
    echo "Cannot find $F1 or $F2, will rebuild ZeroMQ"
fi

if [[ $USE_CACHE == 1 && $ZMQ_IS_BUILT == 1 ]]; then
    echo "Using a cached version of ZeroMQ"
    exit 0
fi


echo "If this fails, make sure that you have installed the packages needed to \
build zmq, for ubuntu and OS X they can be found in\
$WD/install-build-deps-{apt|osx}.sh"
echo ""




### Get the code ###
if [ -e "$WD/$GIT_DIR/.git" ]; then
    (
        cd "$WD/$GIT_DIR"
        git fetch
        git checkout origin/master
    )
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
./configure --prefix="$INSTALL_DIR"    # add other options here
make
make install
set +x

echo "WOOWZ, it's built now. All the good stuff is in $INSTALL_DIR"
