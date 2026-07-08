# PoC to tunnel traffic on MacOS

On way to improve `RAAS` would be to run it on our local machines.
This PoC shows that your MacOS machine can redirect traffic it receives from a client
on the same network (over Wi-Fi) to a tunnel device.

Doing so would allow us to block arbitrary traffic/domains, and perform some interception
where needed.

## How


This consist of 2 stages, executed in sequence.

1. Create the Tunnel device

    ```sh
    ./bootstrap_utun.sh
    ```

2. Note the name of the tunnel device

    ```
    Creating device, requires sudo
    Password:
    2026/06/23 15:20:11 Created: utun4
    ```

3. Configure the Tunnel device

    ```sh
    ./post_up.sh
    ```

    - Enter the `utun<number>` name from step 2
    - Enter the IP of the client that is has its gateway set to your computer

4. Watch the magic happen

    ```sh
    sudo tcpdump -nn -i utun<number>
    ```


