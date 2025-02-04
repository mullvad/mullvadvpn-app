//
//  CustomListRepositoryTests.swift
//  MullvadVPNTests
//
//  Created by Mojgan on 2024-02-07.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadSettings
import Network
import XCTest

class CustomListRepositoryTests: XCTestCase {
    nonisolated(unsafe) static let store = InMemorySettingsStore<SettingNotFound>()
    private var repository = CustomListRepository()

    override class func setUp() {
        SettingsManager.unitTestStore = store
    }

    override class func tearDown() {
        SettingsManager.unitTestStore = nil
    }

    override func tearDownWithError() throws {
        repository.fetchAll().forEach {
            repository.delete(id: $0.id)
        }
    }

    func testFailedAddingDuplicateCustomList() throws {
        let item1 = CustomList(name: "Netflix", locations: [])
        let item2 = CustomList(name: "netflix", locations: [])
        let item3 = CustomList(name: "Netflix", locations: [])

        try XCTAssertNoThrow(repository.save(list: item1))
        try XCTAssertNoThrow(repository.save(list: item2))

        XCTAssertThrowsError(try repository.save(list: item3)) { error in
            XCTAssertEqual(error as? CustomRelayListError, CustomRelayListError.duplicateName)
        }
    }

    func testAddingCustomList() throws {
        let item = CustomList(name: "Netflix", locations: [
            .country("SE"),
            .city("SE", "Gothenburg"),
        ])
        try repository.save(list: item)

        let storedItem = repository.fetch(by: item.id)
        XCTAssertEqual(storedItem, item)
    }

    func testUpdatingCustomList() throws {
        var item = CustomList(name: "Netflix", locations: [
            .country("SE"),
            .city("SE", "Gothenburg"),
        ])
        try repository.save(list: item)

        item.locations.append(.country("FR"))
        try repository.save(list: item)

        let storedItem = repository.fetch(by: item.id)
        XCTAssertEqual(storedItem, item)
    }

    func testDeletingCustomList() throws {
        let item = CustomList(name: "Netflix", locations: [
            .country("SE"),
            .city("SE", "Gothenburg"),
        ])
        try repository.save(list: item)

        let storedItem = repository.fetch(by: item.id)
        repository.delete(id: try XCTUnwrap(storedItem?.id))

        XCTAssertNil(repository.fetch(by: item.id))
    }

    func testFetchingAllCustomList() throws {
        try repository.save(list: CustomList(name: "Netflix", locations: [
            .country("FR"),
            .city("SE", "Gothenburg"),
        ]))

        try repository.save(list: CustomList(name: "PS5", locations: [
            .country("DE"),
            .city("SE", "Gothenburg"),
        ]))

        XCTAssertEqual(repository.fetchAll().count, 2)
    }
}
