//
//  RecentConnectionsRepository.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-10-15.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
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

    public func enable(
        _ selectedEntryConstraint: RelayConstraint<UserSelectedRelays>?,
        selectedExitConstraint: RelayConstraint<UserSelectedRelays>
    ) {
        do {
            // Enable recents with the last selected locations for entry and exit.
            let value = RecentConnections(
                entryLocations: (selectedEntryConstraint != nil) ? [selectedEntryConstraint!] : [],
                exitLocations: [selectedExitConstraint])
            try write(value)
            recentConnectionsSubject.send(.success(value))
        } catch {
            recentConnectionsSubject.send(.failure(error))
        }
    }

    public func add(
        _ selectedEntryConstraint: RelayConstraint<UserSelectedRelays>,
        selectedExitConstraint: RelayConstraint<UserSelectedRelays>
    ) {
        do {
            let current = try read()
            guard current.isEnabled else { throw RecentConnectionsRepositoryError.recentsDisabled }

            let insertAtZero:
                ([RelayConstraint<UserSelectedRelays>], RelayConstraint<UserSelectedRelays>) -> [RelayConstraint<
                    UserSelectedRelays
                >] = { recents, recent in
                    var result: [RelayConstraint<UserSelectedRelays>] = []

                    // Insert the new item first
                    result.append(recent)
                    for item in recents where !self.isDuplicate(result, recent: item) {
                        result.append(item)
                    }
                    return Array(result.prefix(Int(self.maxLimit)))
                }
            let new = RecentConnections(
                entryLocations: insertAtZero(current.entryLocations, selectedEntryConstraint),
                exitLocations: insertAtZero(current.exitLocations, selectedExitConstraint))
            try write(new)
            recentConnectionsSubject.send(.success(new))

        } catch {
            recentConnectionsSubject.send(.failure(error))
        }
    }

    public func load() {
        do {
            let value = try read()
            recentConnectionsSubject.send(.success(value))
        } catch {
            recentConnectionsSubject.send(.failure(error))
        }
    }

    public func deleteCustomList(_ id: UUID) {
        do {
            let current = try read()

            // Clear custom-list selection for items that referenced the deleted ID
            let clearCustomList:
                ([RelayConstraint<UserSelectedRelays>], UUID) -> [RelayConstraint<UserSelectedRelays>] = {
                    recents, id in
                    let new = recents.compactMap { constraint -> RelayConstraint<UserSelectedRelays>? in
                        guard let item = constraint.value else {
                            return nil
                        }

                        guard item.customListSelection?.listId == id else {
                            return .only(item)
                        }

                        let isList = item.customListSelection?.isList ?? false
                        if isList {
                            return nil  // Remove the list
                        }

                        // Keep item but clear the custom list
                        return .only(UserSelectedRelays(locations: item.locations))
                    }
                    return Array(new.prefix(Int(self.maxLimit)))
                }

            // Remove duplicates using the existing isDuplicate logic
            let removeDuplicates: ([RelayConstraint<UserSelectedRelays>]) -> [RelayConstraint<UserSelectedRelays>] = {
                recents in
                var result: [RelayConstraint<UserSelectedRelays>] = []
                for item in recents where !self.isDuplicate(result, recent: item) {
                    result.append(item)
                }
                return result
            }

            let updatedList: ([RelayConstraint<UserSelectedRelays>], UUID) -> [RelayConstraint<UserSelectedRelays>] = {
                recents, id in
                let cleared = clearCustomList(recents, id)  // same call
                let deduped = removeDuplicates(cleared)  // same logic
                return Array(deduped.prefix(Int(self.maxLimit)))  // same limit rule
            }

            let new = RecentConnections(
                entryLocations: updatedList(current.entryLocations, id),
                exitLocations: updatedList(current.exitLocations, id))
            try write(new)
            recentConnectionsSubject.send(.success(new))
        } catch {
            recentConnectionsSubject.send(.failure(error))
        }
    }

    private func isDuplicate(
        _ currentRecents: [RelayConstraint<UserSelectedRelays>], recent: RelayConstraint<UserSelectedRelays>
    ) -> Bool {
        currentRecents.contains { item in
            let isItemList: Bool = item.value?.customListSelection?.isList ?? false
            let isRecentList: Bool = recent.value?.customListSelection?.isList ?? false

            let recent = recent.value
            let item = item.value

            // Both items reference the same custom list (same listId).
            // If both are lists, always treat them as equal (override).
            // Otherwise, remove only when the two items are exactly equal.
            if let recentCustomList = recent?.customListSelection,
                let itemCustomList = item?.customListSelection,
                recentCustomList.listId == itemCustomList.listId
            {
                if isItemList, isRecentList {
                    return true
                }
                return recent == item
            }

            // Neither is a list, locations equal
            if recent?.locations == item?.locations {
                return !(isItemList == true || isRecentList == true)
            }

            // No match
            return false
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
