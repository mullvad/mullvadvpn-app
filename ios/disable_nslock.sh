#!/usr/bin/env bash
set -euo pipefail

usage() {
    echo "Usage: $0 [file path]"
    exit 1
}

if [[ $# -lt 1 ]]; then
    usage
fi

# shellcheck source=ios/configure_paths.sh
source configure_paths.sh

ISSUES=$(rg --column -tswift '(\b(NSRecursiveLock|NSCondition|os_unfair_lock(_t|_lock)?|pthread_mutex_t|DispatchSemaphore)\b)|(\.lock\(\)|\.unlock\(\))\B' "$1" | cut -d' ' -f1)

for issue in ${ISSUES}; do
    echo "${issue}" warning: This usage of locking is not recommended and will soon be a compilation error. Please use withLock constructs or a native synchronisation primitive.
done
exit 0
