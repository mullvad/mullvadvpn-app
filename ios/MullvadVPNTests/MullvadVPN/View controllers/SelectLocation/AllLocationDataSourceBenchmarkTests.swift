//
//  AllLocationDataSourceBenchmarkTests.swift
//  MullvadVPNTests
//
//  Created for performance benchmarking of SelectLocation view.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

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
