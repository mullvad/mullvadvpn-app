//
//  ShadowsocksLoaderTests.swift
//  MullvadVPNTests
//
//  Created by Mojgan on 2024-05-29.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes

import XCTest

class ShadowsocksLoaderTests: XCTestCase {
    private let sampleRelays = ServerRelaysResponseStubs.sampleRelays

    private var relayConstraintsUpdater: RelayConstraintsUpdater!
    private var shadowsocksConfigurationCache: ShadowsocksConfigurationCacheStub!
    private var relaySelector: ShadowsocksRelaySelectorStub!
    private var shadowsocksLoader: ShadowsocksLoader!
    private var relayConstraints = RelayConstraints()

    override func setUpWithError() throws {
        relayConstraintsUpdater = RelayConstraintsUpdater()
        shadowsocksConfigurationCache = ShadowsocksConfigurationCacheStub()
        relaySelector = ShadowsocksRelaySelectorStub(relays: sampleRelays)

        shadowsocksLoader = ShadowsocksLoader(
            cache: shadowsocksConfigurationCache,
            relaySelector: relaySelector,
            constraintsUpdater: relayConstraintsUpdater
        )
    }

    func testLoadConfigWithMultihopDisabled() throws {
        relaySelector.multihopState = .off
        relaySelector.exitBridgeResult = .success(try XCTUnwrap(closetRelayTo(
            location: relayConstraints.exitLocations,
            port: relayConstraints.port,
            filter: relayConstraints.filter,
            in: sampleRelays
        )))
        relaySelector.entryBridgeResult = .failure(ShadowsocksRelaySelectorStubError())

        let configuration = try XCTUnwrap(shadowsocksLoader.load())
        XCTAssertEqual(configuration, try XCTUnwrap(shadowsocksConfigurationCache.read()))
    }

    func testLoadConfigWithMultihopEnabled() throws {
        relaySelector.multihopState = .on
        relaySelector.entryBridgeResult = .success(try XCTUnwrap(closetRelayTo(
            location: relayConstraints.entryLocations,
            port: relayConstraints.port,
            filter: relayConstraints.filter,
            in: sampleRelays
        )))
        relaySelector.exitBridgeResult = .failure(ShadowsocksRelaySelectorStubError())

        let configuration = try XCTUnwrap(shadowsocksLoader.load())
        XCTAssertEqual(configuration, try XCTUnwrap(shadowsocksConfigurationCache.read()))
    }

    func testConstraintsUpdateClearsCache() throws {
        relaySelector.exitBridgeResult = .success(try XCTUnwrap(closetRelayTo(
            location: relayConstraints.exitLocations,
            port: relayConstraints.port,
            filter: relayConstraints.filter,
            in: sampleRelays
        )))
        relaySelector.entryBridgeResult = .failure(ShadowsocksRelaySelectorStubError())

        relayConstraints = RelayConstraints(
            entryLocations: .only(UserSelectedRelays(locations: [.city("ca", "tor")])),
            exitLocations: .only(UserSelectedRelays(locations: [.country("ae")]))
        )

        relayConstraintsUpdater.onNewConstraints?(relayConstraints)

        XCTAssertTrue(shadowsocksConfigurationCache.isCleared)
    }

    private func closetRelayTo(
        location: RelayConstraint<UserSelectedRelays>,
        port: RelayConstraint<UInt16>,
        filter: RelayConstraint<RelayFilter>,
        in: REST.ServerRelaysResponse
    ) -> REST.BridgeRelay? {
        RelaySelector.Shadowsocks.closestRelay(
            location: location,
            port: port,
            filter: filter,
            in: sampleRelays
        )
    }
}

private class ShadowsocksRelaySelectorStub: ShadowsocksRelaySelectorProtocol {
    var entryBridgeResult: Result<REST.BridgeRelay, Error> = .failure(ShadowsocksRelaySelectorStubError())
    var exitBridgeResult: Result<REST.BridgeRelay, Error> = .failure(ShadowsocksRelaySelectorStubError())
    var multihopState: MultihopState = .off

    private let relays: REST.ServerRelaysResponse

    init(relays: REST.ServerRelaysResponse) {
        self.relays = relays
    }

    func selectRelay(with constraints: RelayConstraints) throws -> REST.BridgeRelay? {
        switch multihopState {
        case .on:
            try entryBridgeResult.get()
        case .off:
            try exitBridgeResult.get()
        }
    }

    func getBridges() throws -> REST.ServerShadowsocks? {
        RelaySelector.Shadowsocks.tcpBridge(from: relays)
    }
}

private class ShadowsocksConfigurationCacheStub: ShadowsocksConfigurationCacheProtocol {
    private var cachedConfiguration: ShadowsocksConfiguration?
    private(set) var isCleared = false

    func read() throws -> ShadowsocksConfiguration {
        guard let cachedConfiguration else {
            throw ShadowsocksConfigurationCacheStubError()
        }
        return cachedConfiguration
    }

    func write(_ configuration: ShadowsocksConfiguration) throws {
        self.cachedConfiguration = configuration
    }

    func clear() throws {
        self.cachedConfiguration = nil
        self.isCleared = true
    }
}

private struct ShadowsocksRelaySelectorStubError: Error {}
private struct ShadowsocksConfigurationCacheStubError: Error {}
