# Benchmarks

## Setup

Setup the e2e & benchmark properties in your local `gradle.properties`.

You can specify the ip and port of your target iPerf testing server.
`mullvad.test.benchmark.target.ip=<IP>`
`mullvad.test.benchmark.target.port=<PORT>`

The default values assumes that you have a iPerf server running on a relay bound to wg0 using the default iPerf port.

### Authentication

If no username or password is provided, the tests will not try to authenticate and tests will run without authentication.
If you want to test with authentication set the following properties in your local `gradle.properties`:
`mullvad.test.benchmark.auth.username=<USERNAME>`
`mullvad.test.benchmark.auth.password=<PASSWORD>`

A public key is also required for authentication and is provided in the assets folder of the benchmark module.
It is the public key for the `IPERF_RELAY_ID` relay. If you want to use a different public key you can replace the `iperf3.pem` file in the assets folder with your own public key.

## Run tests

Then start the tests by running the SpeedTests class. The output will be saved in the `test-attachments` folder in the `downloads` folder of the phone.

To manually test iPerf3 on the phone you can use the following command:
`adb shell iperf3 -c <IP> -p <PORT>`
To use authentication you can use the following command:
`IPERF3_PASSWORD=<PASSWORD> && adb shell iperf3 -c <IP> -p <PORT> --rsa-public-key <PUBLIC_KEY> --username <USERNAME> --password <PASSWORD>`
