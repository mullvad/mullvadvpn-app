//
//  RecentConnectionsRepository.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-10-15.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadTypes

public enum RecentConnectionsRepositoryError: LocalizedError, Hashable {
    case recentsDisabled

    public var errorDescription: String? {
        switch self {
        case .recentsDisabled:
            "To add the location to the recents, first enable it in the settings."
        }
    }
}

final public class RecentConnectionsRepository: RecentConnectionsRepositoryProtocol {
    private let store: SettingsStore
    private let maxLimit: UInt
    private let recentConnectionsSubject: PassthroughSubject<RecentConnectionsResult, Never> = .init()

    private let settingsParser: SettingsParser = {
        SettingsParser(decoder: JSONDecoder(), encoder: JSONEncoder())
    }()

    public var recentConnectionsPublisher: AnyPublisher<RecentConnectionsResult, Never> {
        recentConnectionsSubject.eraseToAnyPublisher()
    }

    public init(store: SettingsStore, maxLimit: UInt = 50) {
        self.store = store
        self.maxLimit = maxLimit
    }

    public func disable() {
        do {
            // Clear all recents whenever the recents feature status changes.
            let value = RecentConnections(isEnabled: false, entryLocations: [], exitLocations: [])
            try write(value)
            recentConnectionsSubject.send(.success(value))
        } catch {
            recentConnectionsSubject.send(.failure(error))
        }
    }
    public func enable(_ selectedEntryRelays: UserSelectedRelays?, selectedExitRelays: UserSelectedRelays) {
        do {
            // Enable recents with the last selected locations for entry and exit.
            let value = RecentConnections(
                entryLocations: (selectedEntryRelays != nil) ? [selectedEntryRelays!] : [],
                exitLocations: [selectedExitRelays])
            try write(value)
            recentConnectionsSubject.send(.success(value))
        } catch {
            recentConnectionsSubject.send(.failure(error))
        }
    }

    public func add(_ selectedEntryRelays: UserSelectedRelays, selectedExitRelays: UserSelectedRelays) {
        do {
            let current = try read()
            guard current.isEnabled else { throw RecentConnectionsRepositoryError.recentsDisabled }

            let insertAtZero: ([UserSelectedRelays], UserSelectedRelays) -> [UserSelectedRelays] = { recents, recent in
                var currentRecents = recents

                currentRecents.removeAll(where: { item in
                    let recentCustomList = recent.customListSelection
                    let itemCustomList = item.customListSelection
                    let isCurrentItemListNode: Bool = itemCustomList?.isList ?? false
                    let isRecentItemListNode: Bool = recentCustomList?.isList ?? false

                    // Both have custom lists & same listId
                    if let recentCL = recentCustomList,
                        let itemCL = itemCustomList,
                        recentCL.listId == itemCL.listId
                    {
                        // If both represent the list itself, then replace the old item with the new one.
                        if recentCL.isList && itemCL.isList {
                            return true
                        }
                        // Otherwise remove only if the two entries match exactly
                        return recent == item
                    }

                    // Locations match and neither entry is a list node
                    if recent.locations == item.locations {
                        return isCurrentItemListNode == false && isRecentItemListNode == false
                    }

                    // No match
                    return false
                })
                currentRecents.insert(recent, at: 0)
                return Array(currentRecents.prefix(Int(self.maxLimit)))
            }
            let new = RecentConnections(
                entryLocations: insertAtZero(current.entryLocations, selectedEntryRelays),
                exitLocations: insertAtZero(current.exitLocations, selectedExitRelays))
            try write(new)
            recentConnectionsSubject.send(.success(new))

        } catch {
            recentConnectionsSubject.send(.failure(error))
        }
    }

    public func initiate() {
        do {
            let value = try read()
            recentConnectionsSubject.send(.success(value))
        } catch {
            recentConnectionsSubject.send(.failure(error))
        }
    }

    public func update(_ deletedCustomList: UUID) {
        do {
            let current = try read()
            let removeDuplicates: ([UserSelectedRelays], UUID) -> [UserSelectedRelays] = { recents, id in
                let currentRecents = recents.map({
                    if $0.customListSelection?.listId == id {
                        return UserSelectedRelays(locations: $0.locations)
                    }
                    return $0
                })
                return Array(currentRecents.prefix(Int(self.maxLimit)))
            }
            let new = RecentConnections(
                entryLocations: removeDuplicates(current.entryLocations, deletedCustomList),
                exitLocations: removeDuplicates(current.exitLocations, deletedCustomList))
            try write(new)
            recentConnectionsSubject.send(.success(new))
        } catch {
            recentConnectionsSubject.send(.failure(error))
        }
    }
}

private extension RecentConnectionsRepository {
    private func read() throws -> RecentConnections {
        let data = try store.read(key: .recentConnections)
        return try settingsParser.parseUnversionedPayload(as: RecentConnections.self, from: data)
    }

    private func write(_ value: RecentConnections) throws {
        let data = try settingsParser.produceUnversionedPayload(value)
        try store.write(data, for: .recentConnections)
    }
}
