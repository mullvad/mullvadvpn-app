//
//  TunnelManagerTests.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-02.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import XCTest

@testable import MullvadMockData
@testable import MullvadREST
@testable import MullvadRustRuntime
@testable import MullvadSettings
@testable import MullvadTypes

class TunnelManagerTests: XCTestCase {
    let store = InMemorySettingsStore<SettingNotFound>()
    lazy var settingsManager = SettingsManager(store: store)

    private let application: TunnelStore.BackgroundTaskProvidingObject = UIApplicationStub()
    private var tunnelObserver: TunnelObserver!

    var relayCacheTracker = RelayCacheTrackerStub()
    var accountProxy = AccountsProxyStub()
    var devicesProxy = DevicesProxyStub(
        deviceResult: .success(Device.mock(publicKey: WireGuard.PrivateKey().publicKey))
    )
    var apiProxy = APIProxyStub()
    var apiContext: MullvadApiContext!

    override func setUp() async throws {
        let shadowsocksLoader = ShadowsocksLoader(
            cache: ShadowsocksConfigurationCacheStub(),
            relaySelector: ShadowsocksRelaySelectorStub(relays: .mock()),
            tunnelSettings: LatestTunnelSettings(),
            settingsUpdater: SettingsUpdater(listener: TunnelSettingsListener())
        )

        let opaqueAccessMethodSettingsWrapper = initAccessMethodSettingsWrapper(
            methods: AccessMethodRepositoryStub.stub.fetchAll()
        )

        apiContext = try MullvadApiContext(
            host: REST.defaultAPIHostname,
            address: REST.defaultAPIEndpoint.description,
            domain: REST.encryptedDNSHostname,
            shadowsocksProvider: shadowsocksLoader,
            accessMethodWrapper: opaqueAccessMethodSettingsWrapper,
            accessMethodChangeListeners: []
        )

        try settingsManager.writeSettings(LatestTunnelSettings())
    }

    override func tearDown() async throws {
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
            relaySelector: RelaySelectorStub.nonFallible(),
            settingsManager: settingsManager
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
            relaySelector: RelaySelectorStub.nonFallible(),
            settingsManager: settingsManager
        )
        _ = try await tunnelManager.setNewAccount()
        await tunnelManager.unsetAccount()
        XCTAssertEqual(tunnelManager.isRunningPeriodicPrivateKeyRotation, false)
    }

    /// This test verifies tunnel gets out of `blockedState` after constraints are satisfied.
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
            relaySelector: relaySelector,
            settingsManager: settingsManager
        )

        let simulatorTunnelProviderHost = SimulatorTunnelProviderHost(
            relaySelector: relaySelector,
            apiTransportProvider: APITransportProvider(
                requestFactory: MullvadApiRequestFactory(
                    apiContext: apiContext,
                    encoder: REST.Coding.makeJSONEncoder()
                )
            ),
            settingsManager: settingsManager
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

    /// This test verifies that a refresh tunnel status operation is scheduled whenever the tunnel is being restarted
    func testReconnectingTunnelRefreshesItsStatus() async throws {
        throw XCTSkip("TODO: Fix this flaky test or relieve it of its misery")

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
            relaySelector: relaySelector,
            settingsManager: settingsManager
        )

        let simulatorTunnelProviderHost = SimulatorTunnelProviderHost(
            relaySelector: relaySelector,
            apiTransportProvider: APITransportProvider(
                requestFactory: MullvadApiRequestFactory(
                    apiContext: apiContext,
                    encoder: REST.Coding.makeJSONEncoder()
                )
            ),
            settingsManager: settingsManager
        )

        SimulatorTunnelProvider.shared.delegate = simulatorTunnelProviderHost

        _ = try await tunnelManager.setNewAccount()
        XCTAssertTrue(tunnelManager.deviceState.isLoggedIn)

        let connectedExpectation = expectation(description: "Connected")
        let reconnectingExpectation = expectation(description: "Reconnecting")
        let tunnelObserver = TunnelBlockObserver(
            didUpdateTunnelStatus: { manager, tunnelStatus in
                switch tunnelStatus.state {
                case .connected: connectedExpectation.fulfill()
                case .reconnecting:
                    manager.removeObserver(self.tunnelObserver)
                    reconnectingExpectation.fulfill()
                default: return
                }
            }
        )

        self.tunnelObserver = tunnelObserver
        tunnelManager.addObserver(tunnelObserver)
        tunnelManager.startTunnel()

        await fulfillment(of: [connectedExpectation])

        let reconnectMessageExpectation = expectation(description: "Did witness reconnect message")

        simulatorTunnelProviderHost.onHandleProviderMessage = { message in
            switch message {
            case .reconnectTunnel: reconnectMessageExpectation.fulfill()
            default: break
            }
        }

        tunnelManager.reconnectTunnel(selectNewRelay: false, completionHandler: nil)
        await fulfillment(
            of: [reconnectMessageExpectation, reconnectingExpectation], enforceOrder: true
        )
    }

    /// This test verifies tunnel gets disconnected and reconnected on config reapply.
    func testReapplyingConfigDisconnectsAndReconnects() async throws {
        let connectedExpectation = expectation(description: "Connected!")
        let disconnectedExpectation = expectation(description: "Disconnected!")
        let reconnectedExpectation = expectation(description: "Reconnected!")

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
            relaySelector: relaySelector,
            settingsManager: settingsManager
        )

        let simulatorTunnelProviderHost = SimulatorTunnelProviderHost(
            relaySelector: relaySelector,
            apiTransportProvider: APITransportProvider(
                requestFactory: MullvadApiRequestFactory(
                    apiContext: apiContext,
                    encoder: REST.Coding.makeJSONEncoder()
                )
            ),
            settingsManager: settingsManager
        )
        SimulatorTunnelProvider.shared.delegate = simulatorTunnelProviderHost
        // Observer callbacks are dispatched on the main queue, so this needs no locking.
        var hasConnectedOnce = false
        let tunnelObserver = TunnelBlockObserver(
            didUpdateTunnelStatus: { _, tunnelStatus in
                switch tunnelStatus.state {
                case .connected:
                    if hasConnectedOnce {
                        reconnectedExpectation.fulfill()
                    } else {
                        hasConnectedOnce = true
                        connectedExpectation.fulfill()
                    }
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
        await fulfillment(
            of: [disconnectedExpectation, reconnectedExpectation],
            timeout: .UnitTest.timeout,
            enforceOrder: true
        )
    }
}
