@testable import MullvadREST
import Network
import XCTest

class ServerRelayTests: XCTestCase {
    func testOverrideIPv4AddrIn() throws {
        let overrideRelay: REST.ServerRelay = self.mockServerRelay.override(
            ipv4AddrIn: .loopback,
            ipv6AddrIn: nil
        )

        XCTAssertEqual(overrideRelay.ipv4AddrIn, .loopback)
        XCTAssertEqual(overrideRelay.ipv6AddrIn, .any)
        XCTAssertEqual(overrideRelay.shadowsocksExtraAddrIn, ["\(IPv6Address.any)"])
    }

    func testOverrideIPv6AddrIn() throws {
        let overrideRelay: REST.ServerRelay = self.mockServerRelay.override(
            ipv4AddrIn: nil,
            ipv6AddrIn: .loopback
        )

        XCTAssertEqual(overrideRelay.ipv4AddrIn, .any)
        XCTAssertEqual(overrideRelay.ipv6AddrIn, .loopback)
        XCTAssertEqual(overrideRelay.shadowsocksExtraAddrIn, ["\(IPv4Address.any)"])
    }

    func testOverrideBoth() throws {
        let overrideRelay: REST.ServerRelay = self.mockServerRelay.override(
            ipv4AddrIn: .loopback,
            ipv6AddrIn: .loopback
        )

        XCTAssertEqual(overrideRelay.ipv4AddrIn, .loopback)
        XCTAssertEqual(overrideRelay.ipv6AddrIn, .loopback)
        XCTAssertEqual(overrideRelay.shadowsocksExtraAddrIn, [])
    }

    func testOverrideNone() throws {
        let overrideRelay: REST.ServerRelay = self.mockServerRelay.override(
            ipv4AddrIn: nil,
            ipv6AddrIn: nil
        )

        XCTAssertEqual(overrideRelay.ipv4AddrIn, .any)
        XCTAssertEqual(overrideRelay.ipv6AddrIn, .any)
        XCTAssertEqual(
            overrideRelay.shadowsocksExtraAddrIn,
            shadowSocksExtraAddrIn
        )
    }

    func testOverrideDaita() throws {
        let overrideRelay: REST.ServerRelay = self.mockServerRelay.override(
            daita: true
        )

        XCTAssertEqual(overrideRelay.daita, true)
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
            shadowsocksExtraAddrIn: shadowSocksExtraAddrIn
        )
    }
}
