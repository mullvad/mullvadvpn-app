//
//  DefaultLocationServiceTests.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-10-15.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

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
            relayCache: try MockRelayCache().read().cachedRelays
        )

        let identifier = try await locationService.fetchCurrentLocationIdentifier()

        XCTAssertEqual(identifier?.country, "us")
        XCTAssertEqual(identifier?.city, "dal")
    }
}
