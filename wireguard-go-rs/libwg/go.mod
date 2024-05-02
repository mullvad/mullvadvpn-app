module github.com/mullvad/mullvadvpn-app/wireguard/libwg

go 1.21

require (
	golang.org/x/sys v0.19.0
	golang.zx2c4.com/wireguard v0.0.0-20230223181233-21636207a675
)

require (
	golang.org/x/crypto v0.22.0 // indirect
	golang.org/x/net v0.24.0 // indirect
	golang.zx2c4.com/wintun v0.0.0-20211104114900-415007cec224 // indirect
)

// NOTE: remember to update wireguard-go-rs/Cargo.toml if you change this:
replace golang.zx2c4.com/wireguard => github.com/mullvad/wireguard-go v0.0.0-20240429150257-7cf6da8e40b3
