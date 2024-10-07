module github.com/mullvad/mullvadvpn-app/wireguard/libwg

go 1.21

require (
	golang.org/x/sys v0.19.0
	golang.zx2c4.com/wireguard v0.0.0-20230223181233-21636207a67
)

require (
	github.com/google/btree v1.0.1 // indirect
	golang.org/x/crypto v0.22.0 // indirect
	golang.org/x/net v0.24.0 // indirect
	golang.org/x/time v0.0.0-20220210224613-90d013bbcef8 // indirect
	golang.zx2c4.com/wintun v0.0.0-20230126152724-0fa3db229ce2 // indirect
	gvisor.dev/gvisor v0.0.0-20230927004350-cbd86285d259 // indirect
)

replace golang.zx2c4.com/wireguard => ./wireguard-go
