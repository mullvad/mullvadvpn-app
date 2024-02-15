//
//  CustomListRepositoryTests.swift
//  MullvadVPNTests
//
//  Created by Mojgan on 2024-02-07.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadSettings
import Network
import XCTest

class CustomListRepositoryTests: XCTestCase {
    static let store = InMemorySettingsStore<SettingNotFound>()
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
        let name = "Netflix"
        let item = try XCTUnwrap(repository.create(name, locations: []))
        XCTAssertThrowsError(try repository.create(item.name, locations: [])) { error in
            XCTAssertEqual(error as? CustomRelayListError, CustomRelayListError.duplicateName)
        }
    }

    func testAddingCustomList() throws {
        let name = "Netflix"

        let item = try XCTUnwrap(repository.create(name, locations: [
            .country("SE"),
            .city("SE", "Gothenburg"),
        ]))
        repository.update(item)

        let storedItem = repository.fetch(by: item.id)
        XCTAssertEqual(storedItem, item)
    }

    func testDeletingCustomList() throws {
        let name = "Netflix"

        let item = try XCTUnwrap(repository.create(name, locations: [
            .country("SE"),
            .city("SE", "Gothenburg"),
        ]))
        repository.update(item)

        let storedItem = repository.fetch(by: item.id)
        repository.delete(id: try XCTUnwrap(storedItem?.id))

        XCTAssertNil(repository.fetch(by: item.id))
    }

    func testFetchingAllCustomList() throws {
        let streaming = try XCTUnwrap(repository.create("Netflix", locations: [
            .country("FR"),
            .city("SE", "Gothenburg"),
        ]))
        repository.update(streaming)

        let gaming = try XCTUnwrap(repository.create("PS5", locations: [
            .country("DE"),
            .city("SE", "Gothenburg"),
        ]))
        repository.update(streaming)

        XCTAssertEqual(repository.fetchAll().count, 2)
    }
}
