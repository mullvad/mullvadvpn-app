//
//  TunnelManagerTests.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-02.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
@testable import MullvadSettings
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
        let devicesProxy = DevicesProxyStub()
        let apiProxy = APIProxyStub()
        let accessTokenManager = AccessTokenManagerStub()
        let tunnelManager = TunnelManager(
            application: application,
            tunnelStore: tunnelStore,
            relayCacheTracker: relayCacheTracker,
            accountsProxy: accountProxy,
            devicesProxy: devicesProxy,
            apiProxy: apiProxy,
            accessTokenManager: accessTokenManager
        )
        XCTAssertNotNil(tunnelManager)
    }

    func testLogInStartsKeyRotations() async throws {
        let application = UIApplicationStub()
        let tunnelStore = TunnelStoreStub()
        let relayCacheTracker = RelayCacheTrackerStub()
        var accountProxy = AccountsProxyStub()
        let devicesProxy = DevicesProxyStub()
        let apiProxy = APIProxyStub()
        let accessTokenManager = AccessTokenManagerStub()
        accountProxy.createAccountResult = .success(REST.NewAccountData.mockValue())
        let tunnelManager = TunnelManager(
            application: application,
            tunnelStore: tunnelStore,
            relayCacheTracker: relayCacheTracker,
            accountsProxy: accountProxy,
            devicesProxy: devicesProxy,
            apiProxy: apiProxy,
            accessTokenManager: accessTokenManager
        )
        tunnelManager.startPeriodicPrivateKeyRotation()
        let previousTimer = tunnelManager.nextKeyRotationDate
        _ = try await tunnelManager.setNewAccount()
        XCTAssertNotEqual(previousTimer, tunnelManager.nextKeyRotationDate)
    }

    func testLogOutStopsKeyRotations() async throws {
        let application = UIApplicationStub()
        let tunnelStore = TunnelStoreStub()
        let relayCacheTracker = RelayCacheTrackerStub()
        var accountProxy = AccountsProxyStub()
        let devicesProxy = DevicesProxyStub()
        let apiProxy = APIProxyStub()
        let accessTokenManager = AccessTokenManagerStub()
        accountProxy.createAccountResult = .success(REST.NewAccountData.mockValue())
        let tunnelManager = TunnelManager(
            application: application,
            tunnelStore: tunnelStore,
            relayCacheTracker: relayCacheTracker,
            accountsProxy: accountProxy,
            devicesProxy: devicesProxy,
            apiProxy: apiProxy,
            accessTokenManager: accessTokenManager
        )
        tunnelManager.startPeriodicPrivateKeyRotation()
        await tunnelManager.unsetAccount()
        // This currently fails, as isRunningPeriodicPrivateKeyRotation is not changed.
        XCTAssertEqual(tunnelManager.isRunningPeriodicPrivateKeyRotation, false)
    }
}
