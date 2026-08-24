// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadMockData
import Network
import XCTest

@testable import MullvadREST
@testable import MullvadSettings

final class DestinationDescriberTests: XCTestCase {
    let store = InMemorySettingsStore<SettingNotFound>()

    override func tearDown() {
        store.reset()
    }

    func testDescribeList() throws {
        let relayCacheTracker = MockRelayCacheTracker()
        let customListRepository = CustomListRepository(settingsStore: store)
        let describer = DestinationDescriber(
            relayCacheTracker: relayCacheTracker,
            customListRepository: customListRepository
        )
        let listid = UUID()
        try customListRepository.save(
            list: .init(
                id: listid,
                name: "NameOfList",
                locations: [.country("se"), .country("dk")]
            ))
        XCTAssertEqual(
            describer.describe(
                .init(
                    locations: [.country("se"), .country("dk")],
                    customListSelection: .init(listId: listid, isList: true)
                )),
            "NameOfList"
        )
    }

    func testDescribeSubsetOfList() throws {
        let relayCacheTracker = MockRelayCacheTracker()
        let customListRepository = CustomListRepository(settingsStore: store)
        let describer = DestinationDescriber(
            relayCacheTracker: relayCacheTracker,
            customListRepository: customListRepository
        )
        let listid = UUID()
        try customListRepository.save(
            list: .init(
                id: listid,
                name: "NameOfList2",
                locations: [.country("se"), .country("dk")]
            ))
        XCTAssertEqual(
            describer.describe(
                .init(
                    locations: [.country("se")],
                    customListSelection: .init(listId: listid, isList: false)
                )),
            "Sweden"
        )
    }

    func testDescribeCountryDestination() {
        let relayCacheTracker = MockRelayCacheTracker()
        let customListRepository = CustomListRepository(settingsStore: store)
        let describer = DestinationDescriber(
            relayCacheTracker: relayCacheTracker,
            customListRepository: customListRepository
        )
        XCTAssertEqual(describer.describe(.init(locations: [.country("se")])), "Sweden")
    }

    func testDescribeCityDestination() {
        let relayCacheTracker = MockRelayCacheTracker()
        let customListRepository = CustomListRepository(settingsStore: store)
        let describer = DestinationDescriber(
            relayCacheTracker: relayCacheTracker,
            customListRepository: customListRepository
        )
        XCTAssertEqual(describer.describe(.init(locations: [.city("se", "sto")])), "Stockholm")
    }

    func testDescribeRelayDestination() {
        let relayCacheTracker = MockRelayCacheTracker()
        let customListRepository = CustomListRepository(settingsStore: store)
        let describer = DestinationDescriber(
            relayCacheTracker: relayCacheTracker,
            customListRepository: customListRepository
        )
        XCTAssertEqual(
            describer.describe(.init(locations: [.hostname("se", "sto", "se6-wireguard")])),
            "Stockholm (se6-wireguard)"
        )
    }
}
