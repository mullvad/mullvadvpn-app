// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
