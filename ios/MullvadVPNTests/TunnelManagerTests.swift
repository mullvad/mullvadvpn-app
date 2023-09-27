//
//  TunnelManagerTests.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-02.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import XCTest

final class TunnelManagerTests: XCTestCase {
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
}
