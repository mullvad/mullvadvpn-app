#! /usr/bin/env bash
set -eu

## Everything printed to stdout will be run by travis. So only print
## stuff that needs to be set, e.g. environment variables.

USE_CACHE=${USE_CACHE:-"1"}
SCRIPT_DIR=$(readlink -f $(dirname $0))
INSTALL_DIR=$(readlink -f ${1:-"$SCRIPT_DIR"})

################################################################################
################################## ZMQ #########################################

### TEST IF ZMQ IS BUILT ###
F1=$INSTALL_DIR/zmq/linux/include/zmq.h
F2=$INSTALL_DIR/zmq/linux/lib/libzmq.so
if [[ -f $F1 && -f $F2 ]]; then
    ZMQ_IS_BUILT=1
else
    ZMQ_IS_BUILT=0
fi

if [[ $USE_CACHE == 1 && $ZMQ_IS_BUILT == 1 ]]; then
    >&2 echo "Using a cached version of ZeroMQ"
else
    >&2 echo "Building ZeroMQ"
    >&2 echo ""
    (
        $(dirname $0)/zmq/build-for-linux-with-apt.sh
    ) > stderr

    if [[ "$SCRIPT_DIR" != "$INSTALL_DIR" ]]; then
        mkdir -p $INSTALL_DIR/zmq/linux
        sudo mv ${SCRIPT_DIR}/zmq/linux/* $INSTALL_DIR/zmq/linux/
    fi
fi

>&2 echo "# deps installed, now run"
echo "export LIBZMQ_PREFIX=$INSTALL_DIR/zmq/linux" | tee /dev/stderr
echo "export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$INSTALL_DIR/zmq/linux/lib" | tee /dev/stderr

################################## ZMQ #########################################
################################################################################
