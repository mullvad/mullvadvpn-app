// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import XCTest

@testable import MullvadMockData
@testable import MullvadREST

class DefaultLocationServiceTests: XCTestCase {
    private let encoder = JSONEncoder()

    func testFetchCurrentLocationIdentifier() async throws {
        let mockData = try encoder.encode(
            REST.ServerLocation(
                country: "USA",
                city: "Dallas, TX",
                latitude: 32.89748,
                longitude: -97.040443
            )
        )

        let locationService = DefaultLocationService(
            urlSession: URLSessionStub(
                response: (mockData, URLResponse())
            ),
            relayCache: try MockRelayCache().read()
        )

        let identifier = try await locationService.fetchCurrentLocationIdentifier()

        XCTAssertEqual(identifier?.country, "us")
        XCTAssertEqual(identifier?.city, "dal")
    }
}
