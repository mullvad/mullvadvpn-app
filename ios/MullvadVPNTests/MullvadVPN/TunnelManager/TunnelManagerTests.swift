//
//  TunnelManagerTests.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-02.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//
import MullvadREST

@testable import MullvadMockData
@testable import MullvadSettings
@testable import MullvadTypes
@testable import WireGuardKitTypes

import XCTest

class TunnelManagerTests: XCTestCase {
    static let store = InMemorySettingsStore<SettingNotFound>()
    private var tunnelObserver: TunnelObserver!

    var application: UIApplicationStub!
    var relayCacheTracker: RelayCacheTrackerStub!
    var accountProxy: AccountsProxyStub!
    var accessTokenManager: AccessTokenManagerStub!
    var devicesProxy: DevicesProxyStub!
    var apiProxy: APIProxyStub!

    var transportProvider: TransportProvider!

    override class func setUp() {
        SettingsManager.unitTestStore = store
    }

    override class func tearDown() {
        SettingsManager.unitTestStore = nil
    }

    override func setUp() async throws {
        application = UIApplicationStub()
        relayCacheTracker = RelayCacheTrackerStub()
        accountProxy = AccountsProxyStub()
        accessTokenManager = AccessTokenManagerStub()
        devicesProxy = DevicesProxyStub(deviceResult: .success(Device.mock(publicKey: PrivateKey().publicKey)))
        apiProxy = APIProxyStub()

        transportProvider = TransportProvider(
            urlSessionTransport: URLSessionTransport(urlSession: REST.makeURLSession()),
            addressCache: REST.AddressCache(
                canWriteToCache: true,
                cacheDirectory: FileManager.default.temporaryDirectory
            ),
            transportStrategy: TransportStrategy(
                datasource: AccessMethodRepositoryStub(accessMethods: [PersistentAccessMethod(
                    id: UUID(),
                    name: "direct",
                    isEnabled: true,
                    proxyConfiguration: .direct
                )]),
                shadowsocksLoader: ShadowsocksLoader(
                    cache: ShadowsocksConfigurationCacheStub(),
                    relaySelector: ShadowsocksRelaySelectorStub(relays: .mock()),
                    constraintsUpdater: RelayConstraintsUpdater(),
                    multihopUpdater: MultihopUpdater(listener: MultihopStateListener())
                )
            )
        )

        try SettingsManager.writeSettings(LatestTunnelSettings())
    }

    override func tearDown() async throws {
        application = nil
        relayCacheTracker = nil
        accountProxy = nil
        accessTokenManager = nil
        devicesProxy = nil
        apiProxy = nil
        transportProvider = nil
        tunnelObserver = nil
    }

    func testLogInStartsKeyRotations() async throws {
        accountProxy.createAccountResult = .success(REST.NewAccountData.mockValue())

        let tunnelManager = TunnelManager(
            application: application,
            tunnelStore: TunnelStoreStub(backgroundTaskProvider: application),
            relayCacheTracker: relayCacheTracker,
            accountsProxy: accountProxy,
            devicesProxy: devicesProxy,
            apiProxy: apiProxy,
            accessTokenManager: accessTokenManager,
            relaySelector: RelaySelectorStub.nonFallible()
        )

        _ = try await tunnelManager.setNewAccount()
        XCTAssertEqual(tunnelManager.isRunningPeriodicPrivateKeyRotation, true)
    }

    func testLogOutStopsKeyRotations() async throws {
        accountProxy.createAccountResult = .success(REST.NewAccountData.mockValue())

        let tunnelManager = TunnelManager(
            application: application,
            tunnelStore: TunnelStoreStub(backgroundTaskProvider: application),
            relayCacheTracker: relayCacheTracker,
            accountsProxy: accountProxy,
            devicesProxy: devicesProxy,
            apiProxy: apiProxy,
            accessTokenManager: accessTokenManager,
            relaySelector: RelaySelectorStub.nonFallible()
        )
        _ = try await tunnelManager.setNewAccount()
        await tunnelManager.unsetAccount()
        XCTAssertEqual(tunnelManager.isRunningPeriodicPrivateKeyRotation, false)
    }

    /// This test verifies tunnel gets out of `blockedState` after constraints are satisfied.
    func testExitBlockedStateAfterSatisfyingConstraints() async throws {
        let blockedExpectation = expectation(description: "Relay constraints aren't satisfied!")
        let connectedExpectation = expectation(description: "Connected!")

        accountProxy.createAccountResult = .success(REST.NewAccountData.mockValue())

        let relaySelector = RelaySelectorStub { _, _ in
            try RelaySelectorStub.unsatisfied().selectRelays(with: RelayConstraints(), connectionAttemptCount: 0)
        }

        let tunnelManager = TunnelManager(
            application: application,
            tunnelStore: TunnelStore(application: application),
            relayCacheTracker: relayCacheTracker,
            accountsProxy: accountProxy,
            devicesProxy: devicesProxy,
            apiProxy: apiProxy,
            accessTokenManager: accessTokenManager,
            relaySelector: relaySelector
        )

        let simulatorTunnelProviderHost = SimulatorTunnelProviderHost(
            relaySelector: relaySelector,
            transportProvider: transportProvider
        )
        SimulatorTunnelProvider.shared.delegate = simulatorTunnelProviderHost

        let tunnelObserver = TunnelBlockObserver(
            didUpdateTunnelStatus: { _, tunnelStatus in
                switch tunnelStatus.state {
                case let .error(blockedStateReason) where blockedStateReason == .noRelaysSatisfyingConstraints:
                    blockedExpectation.fulfill()
                    relaySelector.selectedRelaysResult = { relayConstraints, connectionAttemptCount in
                        try RelaySelectorStub.nonFallible().selectRelays(
                            with: relayConstraints,
                            connectionAttemptCount: connectionAttemptCount
                        )
                    }
                    tunnelManager.reconnectTunnel(selectNewRelay: true)

                case .connected:
                    connectedExpectation.fulfill()
                default:
                    return
                }
            }
        )

        self.tunnelObserver = tunnelObserver
        tunnelManager.addObserver(tunnelObserver)

        _ = try await tunnelManager.setNewAccount()

        XCTAssertTrue(tunnelManager.deviceState.isLoggedIn)

        tunnelManager.startTunnel()

        await fulfillment(
            of: [blockedExpectation, connectedExpectation],
            timeout: .UnitTest.timeout,
            enforceOrder: true
        )
    }
}
