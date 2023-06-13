# 

set -o pipefail

VM_NAME=${1:?"No VM name provided"}
SCRIPT=${2:?"No script provided"}
SHARED_DIR=${3:?"No passthrough provided"}

tart run ""--dir=${SHARED_DIR}"" "$VM_NAME" &
vm_pid=$!
ssh admin@$(tart ip "$VM_NAME") zsh /dev/stdin <(echo "$SCRIPT")
script_status=$?

kill $vm_pid
exit $script_status
