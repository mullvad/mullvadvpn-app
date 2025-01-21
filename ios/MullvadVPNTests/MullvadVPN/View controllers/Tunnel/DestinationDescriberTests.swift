//
//  DestinationDescriberTests.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2025-01-21.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
@testable import MullvadREST
@testable import MullvadSettings
import Network
import XCTest

struct MockRelayCache: RelayCacheProtocol {
    func read() throws -> MullvadREST.StoredRelays {
        try .init(
            cachedRelays: CachedRelays(
                relays: .init(
                    locations: [
                        "se-sto-xx-999": .init(
                            country: "Sweden",
                            city: "Stockholm",
                            latitude: 0,
                            longitude: 0),
                    ],
                    wireguard: .init(
                        ipv4Gateway: .any,
                        ipv6Gateway: .any,
                        portRanges: [],
                        relays: [
                            .init(
                                hostname: "se-sto-xx-999",
                                active: true,
                                owned: true,
                                location: "se-sto-xx-999",
                                provider: "",
                                weight: 0,
                                ipv4AddrIn: .any,
                                ipv6AddrIn: .any,
                                publicKey: .init(),
                                includeInCountry: true,
                                daita: nil,
                                shadowsocksExtraAddrIn: nil
                            ),

                        ],
                        shadowsocksPortRanges: []
                    ),
                    bridge: .init(shadowsocks: [], relays: [])
                ),
                updatedAt: Date()
            )
        )
    }

    func readPrebundledRelays() throws -> MullvadREST.StoredRelays {
        try self.read()
    }

    func write(record: MullvadREST.StoredRelays) throws {}
}

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
        try customListRepository.save(list: .init(id: listid, name: "NameOfList", locations: []))
        XCTAssertEqual(
            describer.describe(.init(locations: [], customListSelection: .init(listId: listid, isList: true))),
            "NameOfList"
        )
    }

    func testDescribeCountryDest() {
        let relayCache = MockRelayCache()
        let customListRepository = CustomListRepository()
        let describer = DestinationDescriber(
            relayCache: relayCache,
            customListRepository: customListRepository
        )
        XCTAssertEqual(describer.describe(.init(locations: [.country("se")])), "Sweden")
    }

    func testDescribeCityDest() {
        let relayCache = MockRelayCache()
        let customListRepository = CustomListRepository()
        let describer = DestinationDescriber(
            relayCache: relayCache,
            customListRepository: customListRepository
        )
        XCTAssertEqual(describer.describe(.init(locations: [.city("se", "sto")])), "Stockholm, Sweden")
    }

    func testDescribeRelayDest() {
        let relayCache = MockRelayCache()
        let customListRepository = CustomListRepository()
        let describer = DestinationDescriber(
            relayCache: relayCache,
            customListRepository: customListRepository
        )
        XCTAssertEqual(
            describer.describe(.init(locations: [.hostname("se", "sto", "se-sto-xx-999")])),
            "Stockholm (se-sto-xx-999)"
        )
    }
}
