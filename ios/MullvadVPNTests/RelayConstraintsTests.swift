//
//  RelayConstraintsTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-03-14.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadTypes
import XCTest

final class RelayConstraintsTests: XCTestCase {
    func testMigratingConstraintsFromV1ToLatest() throws {
        let constraintsFromJson = try parseData(from: constraintsV1)
        XCTAssertEqual(constraintsFromJson, latestConstraintsVersion)
    }

    func testMigratingConstraintsFromV2ToLatest() throws {
        let constraintsFromJson = try parseData(from: constraintsV2)
        XCTAssertEqual(constraintsFromJson, latestConstraintsVersion)
    }
}

extension RelayConstraintsTests {
    private var latestConstraintsVersion: RelayConstraints {
        RelayConstraints(
            locations: .only(UserSelectedRelays(locations: [.city("se", "got")])),
            port: .only(80),
            filter: .only(RelayFilter(ownership: .rented, providers: .any))
        )
    }

    private func parseData(from constraintsString: String) throws -> RelayConstraints {
        let data = constraintsString.data(using: .utf8)!
        let decoder = JSONDecoder()

        return try decoder.decode(RelayConstraints.self, from: data)
    }
}

extension RelayConstraintsTests {
    private var constraintsV1: String {
        return """
        {
            "port": {
                "only": 80
            },
            "location": {
                "only": ["se", "got"]
            },
            "filter": {
                "only": {
                    "providers" : "any",
                    "ownership" : {
                        "rented" : {}
                    }
                }
            }
        }
        """
    }

    private var constraintsV2: String {
        return """
        {
            "port": {
                "only": 80
            },
            "locations": {
                "only": {
                    "locations": [["se", "got"]],
                    "customListId": "F17948CB-18E2-4F84-82CD-5780F94216DB"
                }
            },
            "filter": {
                "only": {
                    "providers" : "any",
                    "ownership" : {
                        "rented" : {}
                    }
                }
            }
        }
        """
    }
}
