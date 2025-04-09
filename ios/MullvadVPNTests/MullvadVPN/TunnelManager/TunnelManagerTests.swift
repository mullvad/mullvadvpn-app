//
//  TunnelManagerTests.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-02.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
@testable import MullvadREST

@testable import MullvadMockData
@testable import MullvadSettings
@testable import MullvadTypes
@testable import WireGuardKitTypes

import XCTest

class TunnelManagerTests: XCTestCase {
    static let store = InMemorySettingsStore<SettingNotFound>()
    private var tunnelObserver: TunnelObserver!

    var application: BackgroundTaskProviding!
    var relayCacheTracker: RelayCacheTrackerStub!
    var accountProxy: AccountsProxyStub!
    var accessTokenManager: AccessTokenManagerStub!
    var devicesProxy: DevicesProxyStub!
    var apiProxy: APIProxyStub!
    var addressCache: REST.AddressCache!

    var transportProvider: TransportProvider!

    override static func setUp() {
        SettingsManager.unitTestStore = store
    }

    override static func tearDown() {
        SettingsManager.unitTestStore = nil
    }

    override func setUp() async throws {
        application = UIApplicationStub()
        relayCacheTracker = RelayCacheTrackerStub()
        accountProxy = AccountsProxyStub()
        accessTokenManager = AccessTokenManagerStub()
        devicesProxy = DevicesProxyStub(deviceResult: .success(Device.mock(publicKey: PrivateKey().publicKey)))
        apiProxy = APIProxyStub()
        addressCache = REST.AddressCache(
            canWriteToCache: false,
            fileCache: MockFileCache(initialState: .fileNotFound)
        )

        transportProvider = TransportProvider(
            urlSessionTransport: URLSessionTransport(urlSession: REST.makeURLSession(addressCache: addressCache)),
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
                    settingsUpdater: SettingsUpdater(listener: TunnelSettingsListener())
                )
            ), encryptedDNSTransport: RESTTransportStub()
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
        accountProxy.createAccountResult = .success(NewAccountData.mockValue())

        let tunnelManager = TunnelManager(
            backgroundTaskProvider: application,
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
        accountProxy.createAccountResult = .success(NewAccountData.mockValue())

        let tunnelManager = TunnelManager(
            backgroundTaskProvider: application,
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
    // swiftlint:disable:next function_body_length
    func testExitBlockedStateAfterSatisfyingConstraints() async throws {
        let blockedExpectation = expectation(description: "Relay constraints aren't satisfied!")
        let connectedExpectation = expectation(description: "Connected!")

        accountProxy.createAccountResult = .success(NewAccountData.mockValue())

        let relaySelector = RelaySelectorStub { _ in
            try RelaySelectorStub.unsatisfied().selectRelays(
                tunnelSettings: LatestTunnelSettings(),
                connectionAttemptCount: 0
            )
        }

        let tunnelManager = TunnelManager(
            backgroundTaskProvider: application,
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
            transportProvider: transportProvider,
            apiTransportProvider: APITransportProvider(
                requestFactory: MullvadApiRequestFactory(
                    apiContext: REST.apiContext,
                    encoder: REST.Coding.makeJSONEncoder()
                )
            )
        )
        SimulatorTunnelProvider.shared.delegate = simulatorTunnelProviderHost

        let tunnelObserver = TunnelBlockObserver(
            didUpdateTunnelStatus: { _, tunnelStatus in
                switch tunnelStatus.state {
                case let .error(blockedStateReason) where blockedStateReason == .noRelaysSatisfyingConstraints:
                    blockedExpectation.fulfill()
                    relaySelector.selectedRelaysResult = { connectionAttemptCount in
                        try RelaySelectorStub.nonFallible().selectRelays(
                            tunnelSettings: LatestTunnelSettings(),
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

    /// This test verifies tunnel gets disconnected and reconnected on config reapply.
    func testReapplyingConfigDisconnectsAndReconnects() async throws {
        var connectedExpectation = expectation(description: "Connected!")
        let disconnectedExpectation = expectation(description: "Disconnected!")

        accountProxy.createAccountResult = .success(NewAccountData.mockValue())

        let relaySelector = RelaySelectorStub { _ in
            try RelaySelectorStub.nonFallible().selectRelays(
                tunnelSettings: LatestTunnelSettings(),
                connectionAttemptCount: 0
            )
        }

        let tunnelManager = TunnelManager(
            backgroundTaskProvider: application,
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
            transportProvider: transportProvider,
            apiTransportProvider: APITransportProvider(
                requestFactory: MullvadApiRequestFactory(apiContext: REST.apiContext,
                                                         encoder: REST.Coding.makeJSONEncoder())
            )
        )
        SimulatorTunnelProvider.shared.delegate = simulatorTunnelProviderHost
        let tunnelObserver = TunnelBlockObserver(
            didUpdateTunnelStatus: { _, tunnelStatus in
                switch tunnelStatus.state {
                case .connected: connectedExpectation.fulfill()
                case .disconnected: disconnectedExpectation.fulfill()
                default: return
                }
            }
        )

        self.tunnelObserver = tunnelObserver
        tunnelManager.addObserver(tunnelObserver)

        _ = try await tunnelManager.setNewAccount()

        XCTAssertTrue(tunnelManager.deviceState.isLoggedIn)

        tunnelManager.startTunnel()
        await fulfillment(of: [connectedExpectation])
        tunnelManager.reapplyTunnelConfiguration()
        connectedExpectation = expectation(description: "Connected!")
        await fulfillment(
            of: [disconnectedExpectation, connectedExpectation],
            timeout: .UnitTest.timeout,
            enforceOrder: true
        )
    }
}
