//
//  DestinationDescriberTests.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2025-01-21.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadMockData
@testable import MullvadREST
@testable import MullvadSettings
import Network
import XCTest

final class DestinationDescriberTests: XCTestCase {
    static let store = InMemorySettingsStore<SettingNotFound>()
    override static func setUp() {
        SettingsManager.unitTestStore = store
    }

    override static func tearDown() {
        SettingsManager.unitTestStore = nil
    }

    func testDescribeList() throws {
        let relayCache = MockRelayCache()
        let customListRepository = CustomListRepository()
        let describer = DestinationDescriber(
            relayCache: relayCache,
            customListRepository: customListRepository
        )
        let listid = UUID()
        try customListRepository.save(list: .init(
            id: listid,
            name: "NameOfList",
            locations: [.country("se"), .country("dk")]
        ))
        XCTAssertEqual(
            describer.describe(.init(
                locations: [.country("se"), .country("dk")],
                customListSelection: .init(listId: listid, isList: true)
            )),
            "NameOfList"
        )
    }

    func testDescribeSubsetOfList() throws {
        let relayCache = MockRelayCache()
        let customListRepository = CustomListRepository()
        let describer = DestinationDescriber(
            relayCache: relayCache,
            customListRepository: customListRepository
        )
        let listid = UUID()
        try customListRepository.save(list: .init(
            id: listid,
            name: "NameOfList2",
            locations: [.country("se"), .country("dk")]
        ))
        XCTAssertEqual(
            describer.describe(.init(
                locations: [.country("se")],
                customListSelection: .init(listId: listid, isList: false)
            )),
            "Sweden"
        )
    }

    func testDescribeCountryDestination() {
        let relayCache = MockRelayCache()
        let customListRepository = CustomListRepository()
        let describer = DestinationDescriber(
            relayCache: relayCache,
            customListRepository: customListRepository
        )
        XCTAssertEqual(describer.describe(.init(locations: [.country("se")])), "Sweden")
    }

    func testDescribeCityDestination() {
        let relayCache = MockRelayCache()
        let customListRepository = CustomListRepository()
        let describer = DestinationDescriber(
            relayCache: relayCache,
            customListRepository: customListRepository
        )
        XCTAssertEqual(describer.describe(.init(locations: [.city("se", "sto")])), "Stockholm")
    }

    func testDescribeRelayDestination() {
        let relayCache = MockRelayCache()
        let customListRepository = CustomListRepository()
        let describer = DestinationDescriber(
            relayCache: relayCache,
            customListRepository: customListRepository
        )
        XCTAssertEqual(
            describer.describe(.init(locations: [.hostname("se", "sto", "se6-wireguard")])),
            "Stockholm (se6-wireguard)"
        )
    }
}
