#!/usr/bin/env bash
# This takes the following positional arguments 
# 1. tart VM name
# 2. Script to execute in the VM
# 3. Passthrough directory path, formatted like "$guest_mount_name:$host_dir_path"
#
# The script expects that with the current SSH agent, it's possible to SSH into
# the `admin` user on the VM without any user interaction. The script will
# bring the VM up, execute the specified script via SSH and shut down the VM.
#
# The script returns the exit code of the SSH command.

set -o pipefail

VM_NAME=${1:?"No VM name provided"}
SCRIPT=${2:?"No script path provided"}
PASSTHROUGH_DIR=${3:?"No passthrough directory provided"}

tart run --no-graphics "--dir=${PASSTHROUGH_DIR}" "$VM_NAME" &
vm_pid=$!

# Sleep to wait until VM is up
sleep 10

# apparently, there's a difference between piping into zsh like this and doing
# a <(echo $SCRIPT).
cat "$SCRIPT" | ssh admin@"$(tart ip "$VM_NAME")" bash /dev/stdin
script_status=$?

kill $vm_pid
exit $script_status
