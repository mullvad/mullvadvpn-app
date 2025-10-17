//
//  RecentConnectionsRepository.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-10-15.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
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

final class RecentConnectionsRepository: RecentConnectionsRepositoryProtocol {
    private let store: SettingsStore
    private let maxLimit: UInt

    private let settingsParser: SettingsParser = {
        SettingsParser(decoder: JSONDecoder(), encoder: JSONEncoder())
    }()

    init(store: SettingsStore, maxLimit: UInt = 50) {
        self.store = store
        self.maxLimit = maxLimit
    }

    func setRecentsEnabled(_ isEnabled: Bool) throws {
        // Clear all recents whenever the recents feature status changes.
        try write(RecentConnections(isEnabled: isEnabled, entryLocations: [], exitLocations: []))
    }

    func add(_ location: UserSelectedRelays, as type: RecentLocationType) throws {
        let current = try read()
        guard current.isEnabled else { throw RecentConnectionsRepositoryError.recentsDisabled }
        var currentList = current[keyPath: keyPath(for: type)]
        if let idx = currentList.firstIndex(of: location) { currentList.remove(at: idx) }
        currentList.insert(location, at: 0)
        currentList = Array(currentList.prefix(Int(maxLimit)))

        let new =
            (type == .entry)
            ? RecentConnections(
                isEnabled: current.isEnabled, entryLocations: currentList, exitLocations: current.exitLocations)
            : RecentConnections(
                isEnabled: current.isEnabled, entryLocations: current.entryLocations, exitLocations: currentList)

        try write(new)
    }

    func all() throws -> RecentConnections {
        try read()
    }
}

private extension RecentConnectionsRepository {
    private func keyPath(for type: RecentLocationType) -> KeyPath<RecentConnections, [UserSelectedRelays]> {
        switch type {
        case .entry: return \.entryLocations
        case .exit: return \.exitLocations
        }
    }

    private func read() throws -> RecentConnections {
        let data = try store.read(key: .recentConnections)
        return try settingsParser.parseUnversionedPayload(as: RecentConnections.self, from: data)
    }

    private func write(_ value: RecentConnections) throws {
        let data = try settingsParser.produceUnversionedPayload(value)
        try store.write(data, for: .recentConnections)
    }
}
