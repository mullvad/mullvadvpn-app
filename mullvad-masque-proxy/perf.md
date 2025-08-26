[flamegraph.svg]

The above graph was produced using `perf` while running `iperf3` between
`benchy-alice` and `benchy-bob`. I.e.,

```
# alice
sudo perf record --call-graph dwarf "$masque_server" --cert-path "$cert_pub" --key-path "$cert_key" -b "$MASQUE_SERVER_BIND"

# bob
iperf3 --length "$IPERF_MTU" -c 10.0.1.1 -p "$IPERF_PORT" -t $PERF_DURATION -J
```


```
sudo perf script  | inferno-collapse-perf > stacks.folded
inferno-flamegraph < stacks.folded > profile.svg
```
