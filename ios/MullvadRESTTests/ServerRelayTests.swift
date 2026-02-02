import Network
import XCTest

@testable import MullvadREST

class ServerRelayTests: XCTestCase {
    func testDecodeFromJSON() throws {
        let json = """
            {
                "active": true,
                "hostname": "us-was-wg-002",
                "quic_hostname": "cat.pictures.com",
                "include_in_country": true,
                "ipv4_addr_in": "185.213.193.127",
                "ipv6_addr_in": "2604:980:1002:11::f101",
                "location": "us-was",
                "owned": false,
                "provider": "Zenlayer",
                "public_key": "2AvJGG4MJfnJMRSR6kcha9FZMMkhJM/AtktI5DSESSI=",
                "shadowsocks_extra_addr_in": [
                    "185.213.193.139"
                ],
                "masque_extra_addr_in": [
                    "1.2.3.4",
                    "::1",
                ],
                "stboot": true,
                "weight": 100,
                "daita": true,
                "features": {
                    "daita": {},
                    "quic": {
                        "addr_in": [
                            "45.130.118.209"
                        ],
                        "domain": "se-got-wg-881.blockerad.eu",
                        "token": "test"
                    },
                    "lwo": {}
                }
            }
            """

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let value = try decoder.decode(REST.ServerRelay.self, from: json.data(using: .utf8)!)
        XCTAssertEqual(
            value,
            REST.ServerRelay(
                hostname: "us-was-wg-002",
                active: true,
                owned: false,
                location: .init(rawValue: "us-was")!,
                provider: "Zenlayer",
                weight: 100,
                ipv4AddrIn: IPv4Address("185.213.193.127")!,
                ipv6AddrIn: IPv6Address("2604:980:1002:11::f101")!,
                publicKey: Data(base64Encoded: "2AvJGG4MJfnJMRSR6kcha9FZMMkhJM/AtktI5DSESSI=")!,
                includeInCountry: true,
                daita: true,
                shadowsocksExtraAddrIn: [
                    "185.213.193.139"
                ],
                features: .init(
                    daita: .init(),
                    quic: .init(
                        addrIn: ["45.130.118.209"],
                        domain: "se-got-wg-881.blockerad.eu",
                        token: "test"
                    ),
                    lwo: .init()
                )
            ))
    }

    func testCheckForDaitaWorksFromFeatures() {
        let relayWithDaitaFeature = mockServerRelay.override(features: .init(daita: .init(), quic: nil, lwo: nil))
        let relayWithoutDaitaFeature = mockServerRelay
        XCTAssertTrue(relayWithDaitaFeature.hasDaita)
        XCTAssertFalse(relayWithoutDaitaFeature.hasDaita)
    }

    func testOverrideIPv4AddrIn() throws {
        let overrideRelay: REST.ServerRelay = self.mockServerRelay.override(
            ipv4AddrIn: .loopback,
            ipv6AddrIn: nil
        )

        XCTAssertEqual(overrideRelay.ipv4AddrIn, .loopback)
        XCTAssertEqual(overrideRelay.ipv6AddrIn, .any)
    }

    func testOverrideIPv6AddrIn() throws {
        let overrideRelay: REST.ServerRelay = self.mockServerRelay.override(
            ipv4AddrIn: nil,
            ipv6AddrIn: .loopback
        )

        XCTAssertEqual(overrideRelay.ipv4AddrIn, .any)
        XCTAssertEqual(overrideRelay.ipv6AddrIn, .loopback)
    }

    func testOverrideBoth() throws {
        let overrideRelay: REST.ServerRelay = self.mockServerRelay.override(
            ipv4AddrIn: .loopback,
            ipv6AddrIn: .loopback
        )

        XCTAssertEqual(overrideRelay.ipv4AddrIn, .loopback)
        XCTAssertEqual(overrideRelay.ipv6AddrIn, .loopback)
    }

    func testOverrideNone() throws {
        let overrideRelay: REST.ServerRelay = self.mockServerRelay.override(
            ipv4AddrIn: nil,
            ipv6AddrIn: nil
        )

        XCTAssertEqual(overrideRelay.ipv4AddrIn, .any)
        XCTAssertEqual(overrideRelay.ipv6AddrIn, .any)
    }

    func testOverrideFeatures() throws {
        let overrideRelay: REST.ServerRelay = self.mockServerRelay.override(
            features: REST.ServerRelay.Features(
                daita: REST.ServerRelay.Features.DAITA(),
                quic: REST.ServerRelay.Features.QUIC(addrIn: [""], domain: "", token: ""),
                lwo: REST.ServerRelay.Features.LWO()
            )
        )

        XCTAssertNotNil(overrideRelay.features?.daita)
        XCTAssertNotNil(overrideRelay.features?.quic)
        XCTAssertNotNil(overrideRelay.features?.lwo)
    }

    var shadowSocksExtraAddrIn: [String] {
        ["\(IPv4Address.any)", "\(IPv6Address.any)"]
    }

    var mockServerRelay: REST.ServerRelay {
        REST.ServerRelay(
            hostname: "Host 1",
            active: true,
            owned: true,
            location: "xx-yyy",
            provider: "",
            weight: 0,
            ipv4AddrIn: .any,
            ipv6AddrIn: .any,
            publicKey: Data(),
            includeInCountry: true,
            daita: false,
            shadowsocksExtraAddrIn: shadowSocksExtraAddrIn,
            features: nil
        )
    }
}
