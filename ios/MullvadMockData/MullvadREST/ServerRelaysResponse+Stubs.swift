// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadTypes

@testable import MullvadREST

public enum ServerRelaysResponseStubs {
    public static let wireguardPortRanges: [[UInt16]] = [[4000, 4001], [5000, 5001]]
    public static let shadowsocksPortRanges: [[UInt16]] = [[51900, 51949]]

    /// Loads the prebundled relays.json from MullvadREST bundle for benchmark testing.
    /// This contains real production relay data. The relay list should be updated periodically, especially when new fields are added to it.
    public static func loadPrebundledRelays() throws -> REST.ServerRelaysResponse {
        guard
            let prebundledRelaysFileURL = Bundle(for: RelayCache.self)
                .url(forResource: "relays-test-data", withExtension: "json")
        else {
            throw CocoaError(.fileNoSuchFile)
        }

        let data = try Data(contentsOf: prebundledRelaysFileURL)
        return try REST.Coding.makeJSONDecoder().decode(REST.ServerRelaysResponse.self, from: data)
    }

    /// Returns JSON data based on `sampleRelays` with an additional unknown top-level field
    /// (`"future_feature"`) that `ServerRelaysResponse` doesn't model.
    /// Use this to verify that unknown fields survive round-trips through the cache.
    public static func sampleRelaysJSONWithUnknownField() throws -> Data {
        var json =
            try JSONSerialization.jsonObject(
                with: REST.Coding.makeJSONEncoder().encode(sampleRelays)
            ) as! [String: Any]

        json["future_feature"] = ["key": "value", "nested": [1, 2, 3]] as [String: Any]

        return try JSONSerialization.data(withJSONObject: json)
    }

    /// A digest. Same as `ServerRelaysResponseStubs.sampleRelaysDigest`.
    public static let digest = "2f100d96f4fc831c73f01f51620ea31206da307bdc35137c2a16b984a4027331"

    /// Returns a parseable digest and timestamp. The digest is set to the same value as `ServerRelaysResponseStubs.digest`
    public static let sampleRelaysDigest = """
        {"digest":"\(digest)","timestamp":"2026-09-09T11:44:11+00:00"}

        version=2
        log=c03f05182be9341e33b9edd5f3f8675b08332164640203e743f4285359cace47
        leaf=2acc6e529d374cdb3fc81fbab9af3b739314ae22d1359280f6adf570fa791b8f fe610a1197d9c9b0bfdfffa3e481faf3ca43d860196edb299e54e2b43403298a2ab78cb61c5ff297f736dfd64a482e4035b13d4c4eef83ba019f5e8a4afe9c08

        size=4248
        root_hash=3f4a936ec54da518d7e4af92cce6b5ec9646a6c9f7edf2f22217f00706681988
        signature=b7627d0da0feb2c91963b0c62b65fa8c90ac306930cc69fdc9ac8f9d13ce0986335c52029528a141724c32c397dc9bc064d8660fa416c4db678c30afd101200a
        cosignature=5da2b3803c2f802eed9744b74e3e4a3d31e1e77ed994ef56730d57fa52f698a5 1788954270 8342dd185f12a406828d3b7e3e2431c01ecf47403ce64d547ad39d49bed6911a85ea68824e4659b9647d45832643490e0fe0bd5ad05287d2cf2a6e2e573f7a07
        cosignature=6bdf03b285fce48e00ff9b199cb2b77472dcc4a112f067fa5b274929cb9504e3 1788954270 f31fc919e0f69e98b411857434c7f92937a820f223a8f4f4e41b1cc3db3735a842f26d4306cba3c6da2eec1af4b38269b2017ff1187fa392398e41b6de88370f
        cosignature=774fafee07d3b0d9399d669676440a6301db9fa8fe2140d7d352418da25144c5 1788954270 b6c72618fa0de67cad0bdd8ebb8554e58328f51f7a395c3c34b3e87a26b90f66ca09a76fae5c9e855949d61bd4e537f82fc5c3e661003b19b1a7f9519ecaa103

        leaf_index=4247
        node_hash=bccbe46bd567cf75e195d1a71a54736bb139b61922b91b9fc653e0600eb0e58a
        node_hash=9822608a365a6cd7dd76c07a12b44a4866bc1f16e49377c56270f50c607eb210
        node_hash=5898230443bbb8cf8bbb3d5bc4f4b7f20558a6af710539107a5f6385a87fbca2
        node_hash=209dd8c5ade93675f0d3fae551597ed3e6f4f365d6879e7abd0f084324b1d109
        node_hash=5120276dc8213c1571a9a40b49c0ececdda7309c1b25c376c9645b65370102b8
        node_hash=10a472e685cb0b0af790f2d5068bb0d0ebc928dbe9c0d1e5bbe1a0f0e3c051ae
        """.data(using: .utf8)!

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
            "hr-zag": REST.ServerLocation(
                country: "Croatia",
                city: "Zagreb",
                latitude: 45.821,
                longitude: 15.973
            ),
            "bg-sof": REST.ServerLocation(
                country: "Bulgaria",
                city: "Sofia",
                latitude: 42.683333,
                longitude: 23.316667
            ),
            "gr-ath": REST.ServerLocation(
                country: "Greece",
                city: "Athens",
                latitude: 37.98381,
                longitude: 23.727539
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
                    publicKey: WireGuard.PrivateKey().publicKey.rawValue,
                    includeInCountry: true,
                    daita: true,
                    shadowsocksExtraAddrIn: ["0.0.0.0"],
                    features: .init(daita: .init(), quic: nil, lwo: .init())
                ),
                REST.ServerRelay(
                    hostname: "es2-wireguard",
                    active: true,
                    owned: false,
                    location: "es-mad",
                    provider: "100TB",
                    weight: 500,
                    ipv4AddrIn: .loopback,
                    ipv6AddrIn: .loopback,
                    publicKey: WireGuard.PrivateKey().publicKey.rawValue,
                    includeInCountry: true,
                    daita: true,
                    shadowsocksExtraAddrIn: ["0.0.0.0"],
                    features: .init(daita: .init(), quic: nil, lwo: .init(), lwoV2: .init())
                ),
                REST.ServerRelay(
                    hostname: "es3-wireguard",
                    active: true,
                    owned: false,
                    location: "es-mad",
                    provider: "100TB",
                    weight: 500,
                    ipv4AddrIn: .loopback,
                    ipv6AddrIn: .loopback,
                    publicKey: WireGuard.PrivateKey().publicKey.rawValue,
                    includeInCountry: true,
                    daita: true,
                    shadowsocksExtraAddrIn: ["0.0.0.0"],
                    features: .init(daita: .init(), quic: nil, lwo: nil, lwoV2: .init())
                ),
                REST.ServerRelay(
                    hostname: "es4-wireguard",
                    active: true,
                    owned: false,
                    location: "es-mad",
                    provider: "100TB",
                    weight: 500,
                    ipv4AddrIn: .loopback,
                    ipv6AddrIn: .loopback,
                    publicKey: WireGuard.PrivateKey().publicKey.rawValue,
                    includeInCountry: true,
                    daita: true,
                    shadowsocksExtraAddrIn: ["0.0.0.0"],
                    features: .init(daita: .init(), quic: nil, lwo: nil)
                ),
                REST.ServerRelay(
                    hostname: "es5-wireguard",
                    active: true,
                    owned: false,
                    location: "es-mad",
                    provider: "100TB",
                    weight: 500,
                    ipv4AddrIn: .loopback,
                    ipv6AddrIn: .loopback,
                    publicKey: WireGuard.PrivateKey().publicKey.rawValue,
                    includeInCountry: true,
                    daita: true,
                    shadowsocksExtraAddrIn: ["0.0.0.0"],
                    features: .init(daita: .init(), quic: nil, lwo: nil)
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
                    publicKey: WireGuard.PrivateKey().publicKey.rawValue,
                    includeInCountry: true,
                    daita: false,
                    shadowsocksExtraAddrIn: ["0.0.0.0"],
                    features: .init(
                        daita: nil,
                        quic: .init(addrIn: ["0.0.0.0"], domain: "quic.domain", token: ""),
                        lwo: nil
                    )
                ),
                REST.ServerRelay(
                    hostname: "se3-wireguard",
                    active: true,
                    owned: true,
                    location: "se-got",
                    provider: "100TB",
                    weight: 10,
                    ipv4AddrIn: .loopback,
                    ipv6AddrIn: .loopback,
                    publicKey: WireGuard.PrivateKey().publicKey.rawValue,
                    includeInCountry: true,
                    daita: false,
                    shadowsocksExtraAddrIn: ["::1"],
                    features: .init(
                        daita: nil,
                        quic: .init(addrIn: ["::1"], domain: "quic.domain", token: ""),
                        lwo: nil
                    )
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
                    publicKey: WireGuard.PrivateKey().publicKey.rawValue,
                    includeInCountry: true,
                    daita: false,
                    shadowsocksExtraAddrIn: ["0.0.0.0"],
                    features: nil
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
                    publicKey: WireGuard.PrivateKey().publicKey.rawValue,
                    includeInCountry: true,
                    daita: false,
                    shadowsocksExtraAddrIn: ["0.0.0.0"],
                    features: nil
                ),
                REST.ServerRelay(
                    hostname: "jp1-wireguard",
                    active: true,
                    owned: false,
                    location: "jp-tyo",
                    provider: "100TB",
                    weight: 500,
                    ipv4AddrIn: .loopback,
                    ipv6AddrIn: .loopback,
                    publicKey: WireGuard.PrivateKey().publicKey.rawValue,
                    includeInCountry: true,
                    daita: true,
                    shadowsocksExtraAddrIn: ["0.0.0.0"],
                    features: .init(daita: .init(), quic: nil, lwo: nil)
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
                    publicKey: WireGuard.PrivateKey().publicKey.rawValue,
                    includeInCountry: true,
                    daita: true,
                    shadowsocksExtraAddrIn: ["0.0.0.0"],
                    features: nil
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
                    publicKey: WireGuard.PrivateKey().publicKey.rawValue,
                    includeInCountry: true,
                    daita: true,
                    shadowsocksExtraAddrIn: nil,
                    features: .init(daita: .init(), quic: nil, lwo: nil)
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
                    publicKey: WireGuard.PrivateKey().publicKey.rawValue,
                    includeInCountry: true,
                    daita: true,
                    shadowsocksExtraAddrIn: nil,
                    features: .init(daita: .init(), quic: nil, lwo: nil)
                ),
                REST.ServerRelay(
                    hostname: "hr-zag-wg-001",
                    active: true,
                    owned: false,
                    location: "hr-zag",
                    provider: "DataPacket",
                    weight: 100,
                    ipv4AddrIn: .loopback,
                    ipv6AddrIn: .loopback,
                    publicKey: WireGuard.PrivateKey().publicKey.rawValue,
                    includeInCountry: true,
                    daita: false,
                    shadowsocksExtraAddrIn: nil,
                    features: .init(daita: .init(), quic: nil, lwo: nil)
                ),
                REST.ServerRelay(
                    hostname: "bg-sof-wg-001",
                    active: true,
                    owned: false,
                    location: "bg-sof",
                    provider: "M247",
                    weight: 100,
                    ipv4AddrIn: .loopback,
                    ipv6AddrIn: .loopback,
                    publicKey: WireGuard.PrivateKey().publicKey.rawValue,
                    includeInCountry: true,
                    daita: false,
                    shadowsocksExtraAddrIn: nil,
                    features: .init(daita: .init(), quic: nil, lwo: nil)
                ),
                REST.ServerRelay(
                    hostname: "gr-ath-wg-101",
                    active: true,
                    owned: false,
                    location: "gr-ath",
                    provider: "DataPacket",
                    weight: 100,
                    ipv4AddrIn: .loopback,
                    ipv6AddrIn: .loopback,
                    publicKey: WireGuard.PrivateKey().publicKey.rawValue,
                    includeInCountry: true,
                    daita: false,
                    shadowsocksExtraAddrIn: nil,
                    features: .init(daita: .init(), quic: nil, lwo: nil)
                ),
                REST.ServerRelay(
                    hostname: "us-nyc-wg-101",
                    active: true,
                    owned: false,
                    location: "us-nyc",
                    provider: "DataPacket",
                    weight: 100,
                    ipv4AddrIn: .loopback,
                    ipv6AddrIn: .loopback,
                    publicKey: WireGuard.PrivateKey().publicKey.rawValue,
                    includeInCountry: true,
                    daita: true,
                    shadowsocksExtraAddrIn: nil,
                    features: .init(daita: .init(), quic: nil, lwo: nil)
                ),
            ],
            shadowsocksPortRanges: shadowsocksPortRanges
        ),
        bridge: REST.ServerBridges(
            shadowsocks: [
                REST.ServerShadowsocks(protocol: "tcp", port: 443, cipher: "aes-256-gcm", password: "mullvad")
            ],
            relays: [
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
                    hostname: "se-sto-br-002",
                    active: true,
                    owned: true,
                    location: "se-sto",
                    provider: "31173",
                    ipv4AddrIn: .loopback,
                    weight: 100,
                    includeInCountry: true
                ),
                REST.BridgeRelay(
                    hostname: "se-sto-br-003",
                    active: true,
                    owned: true,
                    location: "se-sto",
                    provider: "31173",
                    ipv4AddrIn: .loopback,
                    weight: 100,
                    includeInCountry: true
                ),
                REST.BridgeRelay(
                    hostname: "se-sto-br-004",
                    active: true,
                    owned: true,
                    location: "se-sto",
                    provider: "31173",
                    ipv4AddrIn: .loopback,
                    weight: 100,
                    includeInCountry: true
                ),
                REST.BridgeRelay(
                    hostname: "se-sto-br-005",
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
