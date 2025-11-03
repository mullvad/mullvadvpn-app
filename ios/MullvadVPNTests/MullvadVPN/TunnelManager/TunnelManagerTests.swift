//
//  TunnelManagerTests.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-02.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import XCTest

@testable import MullvadMockData
@testable import MullvadREST
@testable import MullvadRustRuntime
@testable import MullvadSettings
@testable import MullvadTypes
@testable import WireGuardKitTypes

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
    var apiContext: MullvadApiContext!

    override static func setUp() {
        SettingsManager.unitTestStore = store
    }

    override static func tearDown() {
        store.reset()
    }

    override func setUp() async throws {
        application = UIApplicationStub()
        relayCacheTracker = RelayCacheTrackerStub()
        accountProxy = AccountsProxyStub()
        accessTokenManager = AccessTokenManagerStub()
        devicesProxy = DevicesProxyStub(deviceResult: .success(Device.mock(publicKey: PrivateKey().publicKey)))
        apiProxy = APIProxyStub()
        let shadowsocksLoader = ShadowsocksLoader(
            cache: ShadowsocksConfigurationCacheStub(),
            relaySelector: ShadowsocksRelaySelectorStub(relays: .mock()),
            settingsUpdater: SettingsUpdater(listener: TunnelSettingsListener())
        )
        addressCache = REST.AddressCache(
            canWriteToCache: false,
            fileCache: MockFileCache(initialState: .fileNotFound)
        )
        
        let opaqueAccessMethodSettingsWrapper =  initAccessMethodSettingsWrapper(
            methods: AccessMethodRepositoryStub.stub.fetchAll())

        apiContext = try MullvadApiContext(
            host: REST.defaultAPIHostname,
            address: REST.defaultAPIEndpoint.description,
            domain: REST.encryptedDNSHostname,
            shadowsocksProvider: shadowsocksLoader,
            accessMethodWrapper: opaqueAccessMethodSettingsWrapper,
            addressCacheProvider: addressCache,
            accessMethodChangeListeners: []
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
            relaySelector: relaySelector
        )

        let simulatorTunnelProviderHost = SimulatorTunnelProviderHost(
            relaySelector: relaySelector,
            apiTransportProvider: APITransportProvider(
                requestFactory: MullvadApiRequestFactory(
                    apiContext: apiContext,
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

    /// This test verifies that a refresh tunnel status operation is scheduled whenever the tunnel is being restarted
    func testReconnectingTunnelRefreshesItsStatus() async throws {
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
            relaySelector: relaySelector
        )

        let simulatorTunnelProviderHost = SimulatorTunnelProviderHost(
            relaySelector: relaySelector,
            apiTransportProvider: APITransportProvider(
                requestFactory: MullvadApiRequestFactory(
                    apiContext: apiContext,
                    encoder: REST.Coding.makeJSONEncoder()
                )
            )
        )

        SimulatorTunnelProvider.shared.delegate = simulatorTunnelProviderHost

        _ = try await tunnelManager.setNewAccount()
        XCTAssertTrue(tunnelManager.deviceState.isLoggedIn)

        let connectedExpectation = expectation(description: "Connected")
        let reconnectingExpectation = expectation(description: "Reconnecting")
        let tunnelObserver = TunnelBlockObserver(
            didUpdateTunnelStatus: { _, tunnelStatus in
                switch tunnelStatus.state {
                case .connected: connectedExpectation.fulfill()
                case .reconnecting: reconnectingExpectation.fulfill()
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

        tunnelManager.reconnectTunnel(selectNewRelay: false)
        await fulfillment(
            of: [reconnectMessageExpectation, reconnectingExpectation], enforceOrder: true
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
            relaySelector: relaySelector
        )

        let simulatorTunnelProviderHost = SimulatorTunnelProviderHost(
            relaySelector: relaySelector,
            apiTransportProvider: APITransportProvider(
                requestFactory: MullvadApiRequestFactory(
                    apiContext: apiContext,
                    encoder: REST.Coding.makeJSONEncoder()
                )
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
