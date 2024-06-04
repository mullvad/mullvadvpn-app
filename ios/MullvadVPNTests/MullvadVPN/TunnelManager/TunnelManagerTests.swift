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

final class TunnelManagerTests: XCTestCase {
    static let store = InMemorySettingsStore<SettingNotFound>()

    override class func setUp() {
        SettingsManager.unitTestStore = store
    }

    override class func tearDown() {
        SettingsManager.unitTestStore = nil
    }

    func testTunnelManager() {
        let application = UIApplicationStub()
        let tunnelStore = TunnelStoreStub()
        let relayCacheTracker = RelayCacheTrackerStub()
        let accountProxy = AccountsProxyStub()
        let devicesProxy = DevicesProxyStub(deviceResult: .success(Device.mock(publicKey: PrivateKey().publicKey)))
        let apiProxy = APIProxyStub()
        let accessTokenManager = AccessTokenManagerStub()
        let relaySelector = RelaySelectorStub.nonFallible()
        let tunnelManager = TunnelManager(
            application: application,
            tunnelStore: tunnelStore,
            relayCacheTracker: relayCacheTracker,
            accountsProxy: accountProxy,
            devicesProxy: devicesProxy,
            apiProxy: apiProxy,
            accessTokenManager: accessTokenManager,
            relaySelector: relaySelector
        )
        XCTAssertNotNil(tunnelManager)
    }

    func testLogInStartsKeyRotations() async throws {
        let application = UIApplicationStub()
        let tunnelStore = TunnelStoreStub()
        let relayCacheTracker = RelayCacheTrackerStub()
        var accountProxy = AccountsProxyStub()
        let devicesProxy = DevicesProxyStub(deviceResult: .success(Device.mock(publicKey: PrivateKey().publicKey)))
        let apiProxy = APIProxyStub()
        let accessTokenManager = AccessTokenManagerStub()
        accountProxy.createAccountResult = .success(REST.NewAccountData.mockValue())
        let relaySelector = RelaySelectorStub.nonFallible()
        let tunnelManager = TunnelManager(
            application: application,
            tunnelStore: tunnelStore,
            relayCacheTracker: relayCacheTracker,
            accountsProxy: accountProxy,
            devicesProxy: devicesProxy,
            apiProxy: apiProxy,
            accessTokenManager: accessTokenManager,
            relaySelector: relaySelector
        )
        _ = try await tunnelManager.setNewAccount()
        XCTAssertEqual(tunnelManager.isRunningPeriodicPrivateKeyRotation, true)
    }

    func testLogOutStopsKeyRotations() async throws {
        let application = UIApplicationStub()
        let tunnelStore = TunnelStoreStub()
        let relayCacheTracker = RelayCacheTrackerStub()
        var accountProxy = AccountsProxyStub()
        let devicesProxy = DevicesProxyStub(deviceResult: .success(Device.mock(publicKey: PrivateKey().publicKey)))
        let apiProxy = APIProxyStub()
        let accessTokenManager = AccessTokenManagerStub()
        accountProxy.createAccountResult = .success(REST.NewAccountData.mockValue())
        let relaySelector = RelaySelectorStub.nonFallible()
        let tunnelManager = TunnelManager(
            application: application,
            tunnelStore: tunnelStore,
            relayCacheTracker: relayCacheTracker,
            accountsProxy: accountProxy,
            devicesProxy: devicesProxy,
            apiProxy: apiProxy,
            accessTokenManager: accessTokenManager,
            relaySelector: relaySelector
        )
        _ = try await tunnelManager.setNewAccount()
        await tunnelManager.unsetAccount()
        XCTAssertEqual(tunnelManager.isRunningPeriodicPrivateKeyRotation, false)
    }
}
