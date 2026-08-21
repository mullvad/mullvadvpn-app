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

@testable import MullvadTypes

// There's currently no test for migrating from V2 (RelayConstraint<RelayLocations>) to
// V3 (RelayConstraint<UserSelectedLocations>) due to the only part being changed was an
// optional property. Even if the stored version is V2, the decoder still matches the
// required property of V3 and then disregards the optional, resulting in a successful
// migration. This doesn't affect any end users since during V2 there was no way to
// access the affected features from a release build.
final class RelayConstraintsTests: XCTestCase {
    func testMigratingConstraintsFromV1ToLatest() throws {
        let constraintsFromJson = try parseData(from: constraintsV1)

        let filter: RelayConstraint = .only(RelayFilter(ownership: .rented, providers: .any))
        let constraintsFromInit = RelayConstraints(
            exitLocations: .only(UserSelectedRelays(locations: [.city("se", "got")])),
            port: .only(80),
            entryFilter: filter,
            exitFilter: filter
        )

        XCTAssertEqual(constraintsFromJson, constraintsFromInit)
    }
}

extension RelayConstraintsTests {
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
                        "providers": "any",
                        "ownership": {
                            "rented": {}
                        }
                    }
                }
            }
            """
    }
}
