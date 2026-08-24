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
import MullvadREST
import MullvadTypes
import XCTest

@testable import MullvadSettings

final class AllLocationDataSourceBenchmarkTests: XCTestCase {
    private var prebundledResponse: REST.ServerRelaysResponse!

    override func setUpWithError() throws {
        try super.setUpWithError()
        prebundledResponse = try ServerRelaysResponseStubs.loadPrebundledRelays()
    }

    func testReloadPerformanceWithPrebundledRelays() throws {
        let relays = LocationRelays(
            relays: prebundledResponse.wireguard.relays,
            locations: prebundledResponse.locations
        )
        let dataSource = AllLocationDataSource()

        measure {
            dataSource.reload(relays)
        }
    }

    func testCustomListsDataSourcePerformance() throws {
        // Setup AllLocationDataSource first
        let relays = LocationRelays(
            relays: prebundledResponse.wireguard.relays,
            locations: prebundledResponse.locations
        )
        let allDataSource = AllLocationDataSource()
        allDataSource.reload(relays)

        // Create repository with custom lists referencing multiple locations
        let repository = CustomListsRepositoryStub(customLists: [
            CustomList(name: "Work", locations: [.country("us"), .country("de"), .country("gb")]),
            CustomList(name: "Travel", locations: [.country("jp"), .country("fr"), .country("es"), .country("it")]),
            CustomList(
                name: "Europe",
                locations: [
                    .country("de"), .country("fr"), .country("gb"), .country("es"),
                    .country("it"), .country("nl"), .country("se"), .country("no"),
                ]),
        ])
        let customListsDataSource = CustomListsDataSource(repository: repository)

        measure {
            customListsDataSource.reload(allLocationNodes: allDataSource.nodes)
        }
    }
}
