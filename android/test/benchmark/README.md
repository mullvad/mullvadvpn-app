# Benchmarks

## Setup

Setup the e2e & benchmark properties in your local `gradle.properties`.

Don't forget to specify the IP and port for your target iPerf testing server.
`mullvad.test.benchmark.target.ip=<IP>`
`mullvad.test.benchmark.target.port=<PORT>`

## Run tests

Start the test server and is reachable:
```
iperf3 -s -p <PORT>
```
Make sure the ip and port matches what you've previously configured.

Then start the tests by running the SpeedTests class. The output will be printed in the logs.

