#!/usr/bin/env bash

set -eu

source "$HOME/.cargo/env"

EMAIL_SUBJECT_PREFIX="App test results"
SENDER_EMAIL_ADDR=${SENDER_EMAIL_ADDR-"test@app-test-linux"}
REPORT_ON_SUCCESS=1

if [[ -z "${RECIPIENT_EMAIL_ADDRS+x}" ]]; then
    echo "'RECIPIENT_EMAIL_ADDRS' must be specified" 1>&2
    exit 1
fi

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

mkdir -p "$SCRIPT_DIR/.ci-logs"

rm -f "$SCRIPT_DIR/.ci-logs/last-version.log"
rm -rf "$SCRIPT_DIR/.ci-logs/os"
rm -f "$SCRIPT_DIR/.ci-logs/results.html"

touch "$SCRIPT_DIR/.ci-logs/results.html"

set +e
exec 3>&1
REPORT=$(./ci-runtests.sh $@ 2>&1 | tee >(cat >&3); exit "${PIPESTATUS[0]}")
EXIT_STATUS=$?
set -e

if [[ $REPORT_ON_SUCCESS -eq 0 && $EXIT_STATUS -eq 0 ]]; then
    echo "Not sending email report since tests were successful"
    exit 0
fi

tested_version=$(cat "$SCRIPT_DIR/.ci-logs/last-version.log" || echo "unknown version")

if [[ $EXIT_STATUS -eq 0 ]]; then
    EMAIL_SUBJECT_SUFFIX=" for $tested_version: Succeeded"
else
    EMAIL_SUBJECT_SUFFIX=" for $tested_version: Failed"
fi

echo "Sending email reports"

REPORT_PATH="${SCRIPT_DIR}/.ci-logs/app-testing-$(date +%Y-%m-%d_%H_%M).log"
cat -v - <<<"${REPORT}">"${REPORT_PATH}"

# Attach individual OS logs
attachment_paths=()
for file in $(find "$SCRIPT_DIR/.ci-logs/os" -type f); do
    attachment_paths=("${attachment_paths[@]}" -a "${file}")
done

EMAIL="${SENDER_EMAIL_ADDR}" mutt \
    -e 'set content_type=text/html' \
    -s "${EMAIL_SUBJECT_PREFIX}${EMAIL_SUBJECT_SUFFIX}" \
    -a "${REPORT_PATH}" \
    "${attachment_paths[@]}" \
    -- ${RECIPIENT_EMAIL_ADDRS} <"$SCRIPT_DIR/.ci-logs/results.html"
