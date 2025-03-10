//
//  ServerRelaysResponse+Stubs.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
@testable import MullvadREST
import WireGuardKitTypes

public enum ServerRelaysResponseStubs {
    public static let wireguardPortRanges: [[UInt16]] = [[4000, 4001], [5000, 5001]]
    public static let shadowsocksPortRanges: [[UInt16]] = [[51900, 51949]]

    public static let sampleRelays = REST.ServerRelaysResponse(
        locations: [
            "es-mad": REST.ServerLocation(
                country: "Spain",
                city: "Madrid",
                latitude: 40.408566,
                longitude: -3.69222
            ),
            "se-got": REST.ServerLocation(
                country: "Sweden",
                city: "Gothenburg",
                latitude: 57.70887,
                longitude: 11.97456
            ),
            "se-sto": REST.ServerLocation(
                country: "Sweden",
                city: "Stockholm",
                latitude: 59.3289,
                longitude: 18.0649
            ),
            "ae-dxb": REST.ServerLocation(
                country: "United Arab Emirates",
                city: "Dubai",
                latitude: 25.276987,
                longitude: 55.296249
            ),
            "jp-tyo": REST.ServerLocation(
                country: "Japan",
                city: "Tokyo",
                latitude: 35.685,
                longitude: 139.751389
            ),
            "ca-tor": REST.ServerLocation(
                country: "Canada",
                city: "Toronto",
                latitude: 43.666667,
                longitude: -79.416667
            ),
            "us-atl": REST.ServerLocation(
                country: "USA",
                city: "Atlanta, GA",
                latitude: 40.73061,
                longitude: -73.935242
            ),
            "us-dal": REST.ServerLocation(
                country: "USA",
                city: "Dallas, TX",
                latitude: 32.89748,
                longitude: -97.040443
            ),
            "us-nyc": REST.ServerLocation(
                country: "USA",
                city: "New York, NY",
                latitude: 40.6963302,
                longitude: -74.6034843
            ),
        ],
        wireguard: REST.ServerWireguardTunnels(
            ipv4Gateway: .loopback,
            ipv6Gateway: .loopback,
            portRanges: wireguardPortRanges,
            relays: [
                REST.ServerRelay(
                    hostname: "es1-wireguard",
                    active: true,
                    owned: false,
                    location: "es-mad",
                    provider: "100TB",
                    weight: 500,
                    ipv4AddrIn: .loopback,
                    ipv6AddrIn: .loopback,
                    publicKey: PrivateKey().publicKey.rawValue,
                    includeInCountry: true,
                    daita: true,
                    shadowsocksExtraAddrIn: ["0.0.0.0"]
                ),
                REST.ServerRelay(
                    hostname: "se10-wireguard",
                    active: true,
                    owned: true,
                    location: "se-got",
                    provider: "Blix",
                    weight: 1000,
                    ipv4AddrIn: .loopback,
                    ipv6AddrIn: .loopback,
                    publicKey: PrivateKey().publicKey.rawValue,
                    includeInCountry: true,
                    daita: false,
                    shadowsocksExtraAddrIn: ["0.0.0.0"]
                ),
                REST.ServerRelay(
                    hostname: "se2-wireguard",
                    active: true,
                    owned: true,
                    location: "se-sto",
                    provider: "DataPacket",
                    weight: 50,
                    ipv4AddrIn: .loopback,
                    ipv6AddrIn: .loopback,
                    publicKey: PrivateKey().publicKey.rawValue,
                    includeInCountry: true,
                    daita: false,
                    shadowsocksExtraAddrIn: ["0.0.0.0"]
                ),
                REST.ServerRelay(
                    hostname: "se6-wireguard",
                    active: true,
                    owned: true,
                    location: "se-sto",
                    provider: "31173",
                    weight: 100,
                    ipv4AddrIn: .loopback,
                    ipv6AddrIn: .loopback,
                    publicKey: PrivateKey().publicKey.rawValue,
                    includeInCountry: true,
                    daita: false,
                    shadowsocksExtraAddrIn: ["0.0.0.0"]
                ),
                REST.ServerRelay(
                    hostname: "us-dal-wg-001",
                    active: true,
                    owned: true,
                    location: "us-dal",
                    provider: "M247",
                    weight: 100,
                    ipv4AddrIn: .loopback,
                    ipv6AddrIn: .loopback,
                    publicKey: PrivateKey().publicKey.rawValue,
                    includeInCountry: true,
                    daita: false,
                    shadowsocksExtraAddrIn: ["0.0.0.0"]
                ),
                REST.ServerRelay(
                    hostname: "us-nyc-wg-301",
                    active: true,
                    owned: false,
                    location: "us-nyc",
                    provider: "xtom",
                    weight: 100,
                    ipv4AddrIn: .loopback,
                    ipv6AddrIn: .loopback,
                    publicKey: PrivateKey().publicKey.rawValue,
                    includeInCountry: true,
                    daita: true,
                    shadowsocksExtraAddrIn: nil
                ),
                REST.ServerRelay(
                    hostname: "us-nyc-wg-302",
                    active: false,
                    owned: true,
                    location: "us-nyc",
                    provider: "Qnax",
                    weight: 100,
                    ipv4AddrIn: .loopback,
                    ipv6AddrIn: .loopback,
                    publicKey: PrivateKey().publicKey.rawValue,
                    includeInCountry: true,
                    daita: true,
                    shadowsocksExtraAddrIn: nil
                ),
            ],
            shadowsocksPortRanges: shadowsocksPortRanges
        ),
        bridge: REST.ServerBridges(shadowsocks: [
            REST.ServerShadowsocks(protocol: "tcp", port: 443, cipher: "aes-256-gcm", password: "mullvad"),
        ], relays: [
            REST.BridgeRelay(
                hostname: "se-sto-br-001",
                active: true,
                owned: true,
                location: "se-sto",
                provider: "31173",
                ipv4AddrIn: .loopback,
                weight: 100,
                includeInCountry: true
            ),
            REST.BridgeRelay(
                hostname: "jp-tyo-br-101",
                active: true,
                owned: true,
                location: "jp-tyo",
                provider: "M247",
                ipv4AddrIn: .loopback,
                weight: 1,
                includeInCountry: true
            ),
            REST.BridgeRelay(
                hostname: "ca-tor-ovpn-001",
                active: false,
                owned: false,
                location: "ca-tor",
                provider: "M247",
                ipv4AddrIn: .loopback,
                weight: 1,
                includeInCountry: true
            ),
            REST.BridgeRelay(
                hostname: "ae-dxb-ovpn-001",
                active: true,
                owned: false,
                location: "ae-dxb",
                provider: "M247",
                ipv4AddrIn: .loopback,
                weight: 100,
                includeInCountry: true
            ),
            REST.BridgeRelay(
                hostname: "us-atl-br-101",
                active: true,
                owned: false,
                location: "us-atl",
                provider: "100TB",
                ipv4AddrIn: .loopback,
                weight: 100,
                includeInCountry: true
            ),
            REST.BridgeRelay(
                hostname: "us-dal-br-101",
                active: true,
                owned: false,
                location: "us-dal",
                provider: "100TB",
                ipv4AddrIn: .loopback,
                weight: 100,
                includeInCountry: true
            ),
        ])
    )
}
