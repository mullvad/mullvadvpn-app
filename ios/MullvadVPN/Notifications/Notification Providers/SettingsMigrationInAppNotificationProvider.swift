//
//  SettingsMigrationInAppNotificationProvider.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-05-07.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadSettings
import MullvadTypes
import PacketTunnelCore
import UIKit

final class SettingsMigrationInAppNotificationProvider: NotificationProvider, InAppNotificationProvider,
    @unchecked Sendable
{
    private let tunnelManager: TunnelManager
    private let relaySelector: RelaySelectorProtocol
    private var appPreferences: AppPreferencesDataSource
    private var isSettingsMigrated: Bool
    private var isAppUpdated: Bool
    private var tunnelObserver: TunnelBlockObserver?
    private var migrationOutput: MigrationResult<MultihopStateV2, MultihopSuggestedAction>?
    private var blockedStateReason: BlockedStateReason?
    private let appVersion: String = Bundle.main.productVersion

    private let relayConstraintErrors: [BlockedStateReason] = [
        .noRelaysSatisfyingConstraints,
        .noRelaysSatisfyingFilterConstraints,
        .noRelaysSatisfyingDaitaConstraints,
        .noRelaysSatisfyingObfuscationSettings,
        .noRelaysSatisfyingObfuscationPortConstraints,
        .noRelaysSatisfyingPortConstraints,
    ]

    // Indicates whether settings migration happened during an app update,
    // excluding fresh installs. This is true only when:
    // - settings were migrated to a newer version
    // - the app was updated from a previously installed version
    private var hasMigratedSettings: Bool {
        isSettingsMigrated && isAppUpdated
    }

    // Show the notification when settings were migrated during an app update
    // and the migration flow has not been completed yet.
    private var shouldShowNotification: Bool {
        hasMigratedSettings && appPreferences.hasCompletedMigrationWizard == false
    }

    var notificationDescriptor: InAppNotificationDescriptor? {
        guard shouldShowNotification else { return nil }
        if blockedStateReason != nil,
            migrationOutput?.action != nil
        {
            return InAppNotificationDescriptor(
                identifier: .settingsMigrationInAppNotificationProvider,
                style: .error,
                title: NSLocalizedString("RESOLVE CONNECTION ISSUES", comment: ""),
                body: createNotificationBody(
                    [
                        """
                        Some of your settings have been migrated, please review the changes and resolve any issues.
                        """,
                        "**Tap here to read more**",
                    ].joinedParagraphs(lineBreaks: 1)),
                button: createCloseButtonAction(),
                tapAction: createTapAction())
        } else {
            return InAppNotificationDescriptor(
                identifier: .settingsMigrationInAppNotificationProvider,
                style: .warning,
                title: NSLocalizedString("SETTINGS MIGRATED", comment: ""),
                body: createNotificationBody(
                    [
                        "Your settings have been updated",
                        "**Tap here to read more**",
                    ].joinedParagraphs(lineBreaks: 1)),
                button: createCloseButtonAction(),
                tapAction: createTapAction())
        }
    }

    override var identifier: NotificationProviderIdentifier {
        .settingsMigrationInAppNotificationProvider
    }

    override var priority: NotificationPriority {
        .critical
    }

    init(
        tunnelManager: TunnelManager,
        relaySelector: RelaySelectorProtocol,
        appPreferencesDataSource: AppPreferencesDataSource
    ) {
        self.tunnelManager = tunnelManager
        self.relaySelector = relaySelector
        self.appPreferences = appPreferencesDataSource
        self.isSettingsMigrated = MigratedVersion.current.rawValue > appPreferences.lastMigratedVersion
        self.isAppUpdated =
            appPreferences.lastInstalledVersion.isEmpty ? false : appPreferences.lastInstalledVersion != appVersion
        super.init()

        // Reset the migrated settings menu visibility after an app update.
        // The menu item should only remain visible within the same app version.
        appPreferences.shouldShowMigratedSettingsMenuItem =
            !isAppUpdated && appPreferences.shouldShowMigratedSettingsMenuItem

        appPreferences.lastMigratedVersion = MigratedVersion.current.rawValue
        appPreferences.lastInstalledVersion = appVersion
        addTunnelObserver()
    }

    // MARK: - Private methods
    private func addTunnelObserver() {
        let tunnelObserver = TunnelBlockObserver(
            didLoadConfiguration: { [weak self] tunnelManager in
                guard let self, hasMigratedSettings else { return }
                handleMigratedSettings(tunnelManager.settings)
            },
            didUpdateTunnelStatus: { [weak self] tunnelManager, tunnelStatus in
                guard let self else { return }
                if case .error(let reason) = tunnelStatus.state,
                    relayConstraintErrors.contains(reason)
                {
                    blockedStateReason = reason
                } else if case .connected = tunnelStatus.state {
                    blockedStateReason = nil
                }
                invalidate()
            })
        self.tunnelObserver = tunnelObserver
        tunnelManager.addObserver(tunnelObserver)
    }

    private func handleMigratedSettings(_ settings: LatestTunnelSettings) {
        let tracker = MultihopMigrationTrackerFactory.make(relaySelector)
        let preMigrationSettings = settings
        var copy = settings
        guard let migrationResult = try? tracker.run(input: &copy) else {
            return
        }
        appPreferences.preMigrationSettings = preMigrationSettings
        appPreferences.hasCompletedMigrationWizard = false
        appPreferences.shouldShowMigratedSettingsMenuItem = true
        migrationOutput = migrationResult
        tunnelManager.updateSettings([
            .multihop(migrationResult.value)
        ])
    }

    private func createNotificationBody(_ string: String) -> NSAttributedString {
        NSAttributedString(
            markdownString: string,
            options: MarkdownStylingOptions(font: .mullvadSmall),
            applyEffect: { markdownType, _ in
                guard case .bold = markdownType else { return [:] }
                return [.foregroundColor: UIColor.InAppNotificationBanner.titleColor]
            }
        )
    }

    private func createTapAction() -> InAppNotificationAction {
        InAppNotificationAction(
            handler: { [weak self] in
                guard let self else { return }
                migrationOutput = nil
                isAppUpdated = false
                isSettingsMigrated = false
                invalidate()
                NotificationManager.shared.notificationProvider(self, didReceiveAction: "\(self.identifier)")
            }
        )
    }

    private func createCloseButtonAction() -> InAppNotificationAction {
        InAppNotificationAction(
            image: UIImage.Buttons.closeSmall,
            handler: { [weak self] in
                guard let self else { return }
                migrationOutput = nil
                isAppUpdated = false
                isSettingsMigrated = false
                invalidate()
            }
        )
    }
}
