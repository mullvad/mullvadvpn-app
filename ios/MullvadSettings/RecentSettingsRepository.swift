//
//  RecentSettingsRepository.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-10-15.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadTypes

public enum RecentSettingsError: LocalizedError, Hashable {
    case recentsDisabled

    public var errorDescription: String? {
        switch self {
        case .recentsDisabled:
            "To add the location to the recents first enable it in the settings."
        }
    }
}
final class RecentSettingsRepository: RecentSettingsProtocol {
    private let store: SettingsStore
    private let maxLimit: Int

    private let settingsParser: SettingsParser = {
        SettingsParser(decoder: JSONDecoder(), encoder: JSONEncoder())
    }()

    init(store: SettingsStore, maxLimit: Int = 50) {
        self.store = store
        self.maxLimit = max(0, maxLimit)
    }

    var isRecentConnectionsShown: Bool {
        get throws { try read().isEnabled }
    }

    func setRecentsEnabled(_ isEnabled: Bool) throws {
        try write(RecentSettings(isEnabled: isEnabled, entryLocations: [], exitLocations: []))
    }

    func add(_ location: UserSelectedRelays, to type: RecentLocationType) throws {
        guard try isRecentConnectionsShown else { throw RecentSettingsError.recentsDisabled }
        try update { settings in
            let keyPath = keyPath(for: type)

            let currentList = settings[keyPath: keyPath]
            var list = currentList

            if let idx = currentList.firstIndex(of: location) { list.remove(at: idx) }
            list.insert(location, at: 0)

            list = Array(list.prefix(maxLimit))

            switch type {
            case .entry:
                return RecentSettings(
                    isEnabled: settings.isEnabled,
                    entryLocations: list,
                    exitLocations: settings.exitLocations
                )
            case .exit:
                return RecentSettings(
                    isEnabled: settings.isEnabled,
                    entryLocations: settings.entryLocations,
                    exitLocations: list
                )
            }
        }
    }

    func all() throws -> RecentSettings {
        try read()
    }
}

private extension RecentSettingsRepository {
    private func keyPath(for type: RecentLocationType) -> KeyPath<RecentSettings, [UserSelectedRelays]> {
        switch type {
        case .entry: return \.entryLocations
        case .exit: return \.exitLocations
        }
    }

    private func read() throws -> RecentSettings {
        let data = try store.read(key: .recentSettings)
        return try settingsParser.parseUnversionedPayload(as: RecentSettings.self, from: data)
    }

    private func write(_ value: RecentSettings) throws {
        let data = try settingsParser.produceUnversionedPayload(value)
        try store.write(data, for: .recentSettings)
    }

    func update(_ body: (RecentSettings) -> RecentSettings) throws {
        let current = try read()
        let new = body(current)
        if new != current { try write(new) }
    }
}
