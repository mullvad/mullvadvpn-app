//
//  AmIMullvadTests.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-10-15.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import XCTest

@testable import MullvadMockData
@testable import MullvadREST

class AmIMullvadTests: XCTestCase {
    private let encoder = JSONEncoder()

    func testFetchCurrentLocationIdentifier() async throws {
        let mockData = try encoder.encode(
            REST.ServerLocation(
                country: "Sweden",
                city: "Gothenburg",
                latitude: 57.70887,
                longitude: 11.97456
            )
        )

        let amIMullvad = RESTAmIMullvad(
            urlSession: URLSessionStub(
                response: (mockData, URLResponse())
            ),
            relayCache: try MockRelayCache().read().cachedRelays
        )

        let identifier = try await amIMullvad.fetchCurrentLocationIdentifier()

        XCTAssertEqual(identifier?.country, "se")
        XCTAssertEqual(identifier?.city, "got")
    }

    func testFetchCurrentRelayConstraint() async throws {
        let mockData = try encoder.encode(
            REST.ServerLocation(
                country: "Japan",
                city: "Tokyo",
                latitude: 35.685,
                longitude: 139.751389
            )
        )

        let amIMullvad = RESTAmIMullvad(
            urlSession: URLSessionStub(
                response: (mockData, URLResponse())
            ),
            relayCache: try MockRelayCache().read().cachedRelays
        )

        let constraint = try await amIMullvad.fetchCurrentRelayConstraint()

        XCTAssertEqual(constraint.value?.locations, [.country("jp")])
    }

    func testFetchCurrentRelayConstraintDefaultsToSwedenOnFailure() async throws {
        let mockData = try encoder.encode(
            REST.ServerLocation(
                country: "Nowhere",
                city: "Place",
                latitude: 0,
                longitude: 0
            )
        )

        let amIMullvad = RESTAmIMullvad(
            urlSession: URLSessionStub(
                response: (mockData, URLResponse())
            ),
            relayCache: try MockRelayCache().read().cachedRelays
        )

        let constraint = try await amIMullvad.fetchCurrentRelayConstraint()

        XCTAssertEqual(constraint.value?.locations, [.country("se")])
    }
}
