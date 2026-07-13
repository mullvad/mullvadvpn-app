//
//  DeprecatedSettingsResolver.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-06-10.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import MullvadLogging
import MullvadREST
import MullvadSettings
import MullvadTypes

enum DeprecatedSettingsResolverResult: Sendable {

    // Nothing to migrate
    case nothing

    /// Successfully performed migration.
    case migrated(from: LatestTunnelSettings, to: LatestTunnelSettings, changes: [Change])

    /// Failure when migrating store.
    case failure(Error)
}

struct DeprecatedSettingsResolver: Sendable {
    private let cacheDirectory: URL
    private let settingsManager: SettingsManager
    private let relaySelector: RelaySelectorProtocol
    private let currentVersion: MigratedVersion

    public init(
        cacheDirectory: URL,
        settingsManager: SettingsManager,
        relaySelector: RelaySelectorProtocol,
        currentVersion: MigratedVersion
    ) {
        self.cacheDirectory = cacheDirectory.appendingPathComponent("migrationState.json")
        self.settingsManager = settingsManager
        self.relaySelector = relaySelector
        self.currentVersion = currentVersion
    }

    public func resolve(
        store: SettingsStore,
        migrationCompleted: @escaping @Sendable (DeprecatedSettingsResolverResult) -> Void
    ) {

        let fileCoordinator = NSFileCoordinator(filePresenter: nil)
        var error: NSError?

        // This will block the calling thread if another process is currently running the same code.
        // This is intentional to avoid TOCTOU issues, and guaranteeing settings cannot be read
        // in a half written state.
        // The resulting effect is that only one process at a time can do settings migrations.
        // The other process will be blocked, and will have nothing to do as long as settings were successfully upgraded.
        fileCoordinator.coordinate(writingItemAt: cacheDirectory, error: &error) { _ in
            do {
                try migrateSettings(store: store, migrationCompleted: migrationCompleted)
            } catch {
                migrationCompleted(.failure(error))
            }
        }
    }

    private func migrateSettings(
        store: SettingsStore,
        migrationCompleted: @escaping @Sendable (DeprecatedSettingsResolverResult) -> Void
    ) throws {
        guard currentVersion != MigratedVersion.current else {
            migrationCompleted(.nothing)
            return
        }

        let parser = SettingsParser(decoder: JSONDecoder(), encoder: JSONEncoder())
        let settingsData = try store.read(key: SettingsKey.settings)

        do {
            let currentSettings = try parser.parsePayload(as: LatestTunnelSettings.self, from: settingsData)
            var copy = currentSettings
            let migrationOutput = try MultihopMigrationTrackerFactory.make(relaySelector).run(input: &copy)
            migrationCompleted(.migrated(from: currentSettings, to: copy, changes: migrationOutput.changes))
        } catch {
            migrationCompleted(.failure(error))
        }
    }
}
