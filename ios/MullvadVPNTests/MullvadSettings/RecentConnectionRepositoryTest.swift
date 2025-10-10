//
//  RecentConnectionRepositoryTest.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-09-25.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import XCTest

@testable import MullvadSettings
@testable import MullvadTypes

final class RecentConnectionRepositoryTests: XCTestCase {
    private var store: InMemorySettingsStore<SettingNotFound>!
    override func setUpWithError() throws {
        try super.setUpWithError()
        store = InMemorySettingsStore<SettingNotFound>()
        SettingsManager.unitTestStore = store
    }

    override func tearDownWithError() throws {
        try super.tearDownWithError()
        store.reset()
    }

    func testAddWithDefaultLimit() throws {
        let recentConnections = [
            RecentConnection(
                entry: UserSelectedRelays(locations: [.country("se")]),
                exit: UserSelectedRelays(locations: [.city("al", "tia")])
            )
        ]

        let repository = RecentConnectionRepository()
        try repository.clear()
        for recentConnection in recentConnections {
            try repository.add(recentConnection)
        }

        let all = try repository.all()
        XCTAssertEqual(all.count, recentConnections.count, "Should store all added items up to default limit.")
    }

    func testExceedLimitAndDeletesOldest() throws {
        let maxLimit = 3
        let recentConnections: [RecentConnection] = [
            RecentConnection(
                entry: .init(locations: [.country("se")]),
                exit: .init(locations: [.city("al", "tia")])
            ),
            RecentConnection(
                entry: .init(locations: [.country("se")]),
                exit: .init(locations: [.country("fr")])
            ),
            RecentConnection(
                entry: .init(locations: [.country("de")]),
                exit: .init(locations: [.country("nl")])
            ),
            RecentConnection(exit: .init(locations: [.country("nl")])),
        ]

        let repository = RecentConnectionRepository(maxLimit: 3)
        try repository.clear()

        for recentConnection in recentConnections {
            try repository.add(recentConnection)
        }

        let all = try repository.all()
        XCTAssertEqual(all.count, maxLimit, "Should be equal to max limit.")
        XCTAssertFalse(all.contains(recentConnections.first!), "Oldest item should be removed when exceeding limit.")
    }

    func testAvoidSavingDuplicate() throws {
        let params: [RecentConnection] = [
            RecentConnection(exit: .init(locations: [.country("se")])),
            RecentConnection(exit: .init(locations: [.country("sp")])),
            RecentConnection(exit: .init(locations: [.country("se")])),
            RecentConnection(
                entry: .init(locations: [.country("se")]),
                exit: .init(locations: [.country("nl")])
            ),
        ]

        let repository = RecentConnectionRepository()
        try repository.clear()
        for recentConnection in params {
            try repository.add(recentConnection)
        }

        let all = try repository.all()
        XCTAssertEqual(all.count, params.count - 1, "Should not store duplicate RecentConnection items.")
    }

    func testRemovesAll() throws {
        let repository = RecentConnectionRepository()
        try repository.clear()
        let all = try repository.all()
        XCTAssertTrue(all.isEmpty, "Clear should remove all RecentConnection items.")
    }
}
