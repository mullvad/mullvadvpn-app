#! /usr/bin/env bash

set -eux

apt-get update
apt-get -y install git build-essential libtool autoconf automake uuid-dev pkg-config
