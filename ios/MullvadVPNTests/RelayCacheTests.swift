//
//  RelayCacheTests.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-06-02.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
import MullvadSettings
import Network
import XCTest

final class RelayCacheTests: XCTestCase {
    func testReadCache() throws {
        let fileCache = MockFileCache(
            initialState: .exists(CachedRelays(relays: .mock(), updatedAt: .distantPast))
        )
        let cache = RelayCache(fileCache: fileCache, ipOverrideRepository: IPOverrideRepositoryStub())
        let relays = try XCTUnwrap(cache.read())

        XCTAssertEqual(fileCache.getState(), .exists(relays))
    }

    func testWriteCache() throws {
        let fileCache = MockFileCache(
            initialState: .exists(CachedRelays(relays: .mock(), updatedAt: .distantPast))
        )
        let cache = RelayCache(fileCache: fileCache, ipOverrideRepository: IPOverrideRepositoryStub())
        let newCachedRelays = CachedRelays(relays: .mock(), updatedAt: Date())

        try cache.write(record: newCachedRelays)
        XCTAssertEqual(fileCache.getState(), .exists(newCachedRelays))
    }

    func testReadPrebundledRelaysWhenNoCacheIsStored() throws {
        let fileCache = MockFileCache<CachedRelays>(initialState: .fileNotFound)
        let cache = RelayCache(fileCache: fileCache, ipOverrideRepository: IPOverrideRepositoryStub())

        XCTAssertNoThrow(try cache.read())
    }

    func testOverrideServerRelayInCache() throws {
        let relays = [
            mockServerRelay.copyWith(ipv4AddrIn: .loopback, ipv6AddrIn: .broadcast),
            mockServerRelay,
        ]

        let fileCache = MockFileCache(
            initialState: .exists(CachedRelays(relays: .mock(serverRelays: relays), updatedAt: .distantPast))
        )

        let override = try IPOverride(hostname: "Host 1", ipv4Address: .loopback, ipv6Address: .broadcast)

        let cache = RelayCache(
            fileCache: fileCache,
            ipOverrideRepository: IPOverrideRepositoryStub(overrides: [override])
        )

        let storedCache = try cache.read()

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
            mockBridgeRelay.copyWith(ipv4AddrIn: .loopback),
            mockBridgeRelay,
        ]

        let fileCache = MockFileCache(
            initialState: .exists(CachedRelays(relays: .mock(brideRelays: relays), updatedAt: .distantPast))
        )

        let override = try IPOverride(hostname: "Host 1", ipv4Address: .loopback, ipv6Address: .broadcast)

        let cache = RelayCache(
            fileCache: fileCache,
            ipOverrideRepository: IPOverrideRepositoryStub(overrides: [override])
        )

        let storedCache = try cache.read()

        // Assert that relay was overridden.
        let host1 = storedCache.relays.bridge.relays.first
        XCTAssertEqual(host1?.ipv4AddrIn, .loopback)

        // Assert that relay was NOT overridden.
        let host2 = storedCache.relays.bridge.relays.last
        XCTAssertEqual(host2?.ipv4AddrIn, .any)
    }
}

extension RelayCacheTests {
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

private extension REST.ServerRelaysResponse {
    static func mock(
        serverRelays: [REST.ServerRelay] = [],
        brideRelays: [REST.BridgeRelay] = []
    ) -> Self {
        REST.ServerRelaysResponse(
            locations: [:],
            wireguard: REST.ServerWireguardTunnels(
                ipv4Gateway: .loopback,
                ipv6Gateway: .loopback,
                portRanges: [],
                relays: serverRelays
            ),
            bridge: REST.ServerBridges(shadowsocks: [], relays: brideRelays)
        )
    }
}
