#!/usr/bin/env bash

echo "Updating relay list..."
set +e
read -d '' JSONRPC_CODE <<-JSONRPC_CODE
var buff = "";
process.stdin.on('data', function (chunk) {
    buff += chunk;
})
process.stdin.on('end', function () {
    var obj = JSON.parse(buff);
    var output = JSON.stringify(obj.result, null, '    ');
    process.stdout.write(output);
})
JSONRPC_CODE
set -e

JSONRPC_RESPONSE="$(curl -X POST \
    --fail \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc": "2.0", "id": "0", "method": "relay_list_v3"}' \
     https://api.mullvad.net/rpc/)"
echo $JSONRPC_RESPONSE | node -e "$JSONRPC_CODE" >  dist-assets/relays.json
