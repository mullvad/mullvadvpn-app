#! /usr/bin/env bash
set -eu

## Everything printed to stdout will be run by travis. So only print
## stuff that needs to be set, e.g. environment variables.

SCRIPT_DIR=$(readlink -f $(dirname $0))
INSTALL_DIR=$(readlink -f ${1:-"$SCRIPT_DIR"})

################################################################################
################################## ZMQ #########################################

mkdir -p $INSTALL_DIR/zmq/unix
$(dirname $0)/zmq/build.sh $INSTALL_DIR/zmq/unix >&2

echo "# ZeroMQ is installed, now run" >&2
echo "export LIBZMQ_PREFIX=$INSTALL_DIR/zmq/unix" | tee /dev/stderr
set +u
echo "export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$INSTALL_DIR/zmq/unix/lib" | tee /dev/stderr
set -u

################################## ZMQ #########################################
################################################################################
