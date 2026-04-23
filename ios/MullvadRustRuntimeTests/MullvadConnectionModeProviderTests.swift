//
//  MullvadConnectionModeProviderTests.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-04-27.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
import MullvadRustRuntime
import MullvadTypes
import XCTest

class MullvadConnectionModeProviderTests: XCTestCase {
    func testInvalidCipherDoesNotCauseExceptionInRust() {
        var methods = AccessMethodRepositoryStub.stub.fetchAll()

        let customMethod = PersistentAccessMethod(
            id: UUID(),
            name: "Method",
            isEnabled: true,
            proxyConfiguration: .shadowsocks(
                PersistentProxyConfiguration.ShadowsocksConfiguration(
                    server: .ipv4(.loopback),
                    port: 1,
                    password: "",
                    cipher: "invalidCipher"
                )
            )
        )

        methods.append(customMethod)

        _ = initAccessMethodSettingsWrapper(methods: methods)
        XCTAssert(true)
    }
}
