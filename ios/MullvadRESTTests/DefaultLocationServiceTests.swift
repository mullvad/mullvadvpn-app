// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Network
import XCTest

@testable import MullvadMockData
@testable import MullvadREST
@testable import MullvadRustRuntime
@testable import MullvadTypes

class DefaultLocationServiceTests: XCTestCase {
    private let encoder = JSONEncoder()

    func testFetchCurrentLocationIdentifier() async throws {
        let mockData = AmIMullvadResponse(
            ip: .ipv4(.loopback),
            country: "USA",
            city: "Dallas, TX",
            latitude: 32.89748,
            longitude: -97.040443,
            mullvadExitIp: false,
        )

        let locationService = DefaultLocationService(
            relayCache: try MockRelayCache().read(),
            apiContext: try makeApiContext(),
        )

        let identifier = locationService.getLocation(mockData)

        XCTAssertEqual(identifier?.country, "us")
        XCTAssertEqual(identifier?.city, "dal")
    }

    private func makeApiContext() throws -> ApiContext {
        let shadowsocksLoader = ShadowsocksLoaderStub(
            configuration: ShadowsocksConfiguration(
                address: .ipv4(.loopback),
                port: 1080,
                password: "123",
                cipher: "aes-128-cfb"
            ))

        let accessMethodsRepository = AccessMethodRepositoryStub.stub

        return ApiContext(
            host: "localhost",
            address: REST.defaultAPIEndpoint.description,
            domain: REST.encryptedDNSHostname,
            disableTls: true,
            shadowsocksProvider: shadowsocksLoader,
            accessMethodWrapper: initAccessMethodSettingsWrapper(methods: accessMethodsRepository.fetchAll()),
            accessMethodChangeListeners: []
        )
    }
}
