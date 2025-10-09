//
//  RecentConnectionRepositoryTest.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-09-25.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import Testing

@testable import MullvadSettings
@testable import MullvadTypes

@Suite("RecentConnectionRepositoryTest")
struct RecentConnectionRepositoryTest {
    enum Scenario: Sendable, CustomTestStringConvertible {
        case add([RecentConnection])
        case exceedLimit([RecentConnection], max: Int)
        case addDuplicate([RecentConnection])
        case cleanup

        var testDescription: String {
            switch self {
            case .add: "Adds items up to the default limit (50)"
            case .exceedLimit: "Enforces a max limit by deleting the oldest"
            case .addDuplicate: "Should not store duplicate RecentConnection items"
            case .cleanup: "Clear removes all RecentConnection items"
            }
        }
    }

    @Test(
        .serialized,
        arguments: [
            Scenario.add([
                RecentConnection(
                    entry: UserSelectedRelays(locations: [.country("se")]),
                    exit: UserSelectedRelays(locations: [.city("al", "tia")])
                )
            ]),
            Scenario.exceedLimit(
                [
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
                ], max: 2),
            Scenario.addDuplicate([
                RecentConnection(exit: .init(locations: [.country("se")])),
                RecentConnection(exit: .init(locations: [.country("sp")])),
                RecentConnection(exit: .init(locations: [.country("se")])),
                RecentConnection(
                    entry: .init(locations: [.country("se")]),
                    exit: .init(locations: [.country("nl")])
                ),
            ]),
            Scenario.cleanup,
        ]
    )
    func run(_ scenario: Scenario) {
        let store = InMemorySettingsStore<SettingNotFound>()
        SettingsManager.unitTestStore = store

        switch scenario {
        case let .add(recentConnections):
            let repository = RecentConnectionRepository()
            repository.clear()
            for recentConnection in recentConnections { repository.add(recentConnection) }
            let all = repository.all()
            #expect(all.count == recentConnections.count)

        case let .exceedLimit(recentConnections, max):
            let repository = RecentConnectionRepository(maxLimit: max)
            repository.clear()
            for recentConnection in recentConnections { repository.add(recentConnection) }

            let all = repository.all()
            #expect(all.count == max)
            #expect(all.contains(recentConnections.first!) == false)

        case let .addDuplicate(params):
            let repository = RecentConnectionRepository()
            repository.clear()
            for recentConnection in params { repository.add(recentConnection) }
            let all = repository.all()
            #expect(all.count == params.count - 1)

        case .cleanup:
            let repository = RecentConnectionRepository()
            repository.clear()
            let all = repository.all()
            #expect(all.isEmpty)
        }
    }
}
