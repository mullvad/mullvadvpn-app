#!/bin/bash

set -eu

# The profile link already exists, skip this step
if [[ -f "$HOME/.profile" ]]
then
    exit 0
fi

touch ~/.zprofile
ln -s ~/.zprofile ~/.profile