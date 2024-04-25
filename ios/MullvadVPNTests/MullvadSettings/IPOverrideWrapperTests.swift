//
//  IPOverrideWrapperTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-02-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
import MullvadSettings
import Network
import XCTest

final class IPOverrideWrapperTests: XCTestCase {
    func testOverrideServerRelayInCache() throws {
        let relays = [
            mockServerRelay.override(ipv4AddrIn: .loopback, ipv6AddrIn: .broadcast),
            mockServerRelay,
        ]

        let fileCache = MockFileCache(
            initialState: .exists(CachedRelays(relays: .mock(serverRelays: relays), updatedAt: .distantPast))
        )

        let override = try IPOverride(hostname: "Host 1", ipv4Address: .loopback, ipv6Address: .broadcast)

        let overrideWrapper = IPOverrideWrapper(
            relayCache: RelayCache(fileCache: fileCache),
            ipOverrideRepository: IPOverrideRepositoryStub(overrides: [override])
        )

        let storedCache = try overrideWrapper.read()

        // Assert that relay was overridden.
        let host1 = storedCache.relays.wireguard.relays.first
        XCTAssertEqual(host1?.ipv4AddrIn, .loopback)
        XCTAssertEqual(host1?.ipv6AddrIn, .broadcast)

        // Assert that relay was NOT overridden.
        let host2 = storedCache.relays.wireguard.relays.last
        XCTAssertEqual(host2?.ipv4AddrIn, .any)
        XCTAssertEqual(host2?.ipv6AddrIn, .any)
    }

    func testOverrideBridgeRelayInCache() throws {
        let relays = [
            mockBridgeRelay.override(ipv4AddrIn: .loopback),
            mockBridgeRelay,
        ]

        let fileCache = MockFileCache(
            initialState: .exists(CachedRelays(relays: .mock(brideRelays: relays), updatedAt: .distantPast))
        )

        let override = try IPOverride(hostname: "Host 1", ipv4Address: .loopback, ipv6Address: .broadcast)

        let overrideWrapper = IPOverrideWrapper(
            relayCache: RelayCache(fileCache: fileCache),
            ipOverrideRepository: IPOverrideRepositoryStub(overrides: [override])
        )

        let storedCache = try overrideWrapper.read()

        // Assert that relay was overridden.
        let host1 = storedCache.relays.bridge.relays.first
        XCTAssertEqual(host1?.ipv4AddrIn, .loopback)

        // Assert that relay was NOT overridden.
        let host2 = storedCache.relays.bridge.relays.last
        XCTAssertEqual(host2?.ipv4AddrIn, .any)
    }
}

extension IPOverrideWrapperTests {
    var mockServerRelay: REST.ServerRelay {
        REST.ServerRelay(
            hostname: "",
            active: true,
            owned: true,
            location: "",
            provider: "",
            weight: 0,
            ipv4AddrIn: .any,
            ipv6AddrIn: .any,
            publicKey: Data(),
            includeInCountry: true
        )
    }

    var mockBridgeRelay: REST.BridgeRelay {
        REST.BridgeRelay(
            hostname: "",
            active: true,
            owned: true,
            location: "",
            provider: "",
            ipv4AddrIn: .any,
            weight: 0,
            includeInCountry: true
        )
    }
}
