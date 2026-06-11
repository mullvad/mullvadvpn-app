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
    private let appVersion: String = Bundle.main.productVersion
    private let tunnelManager: TunnelManager
    private var migratedSettingsState: MigratedSettingsState?
    private var migratedSettingsUpdater: MigratedSettingsUpdater

    private var tunnelObserver: TunnelBlockObserver?
    private var migratedSettingsObserverBlock: MigratedSettingsObserverBlock?
    private var blockedStateReason: BlockedStateReason?

    private let relayConstraintErrors: [BlockedStateReason] = [
        .noRelaysSatisfyingConstraints,
        .noRelaysSatisfyingFilterConstraints,
        .noRelaysSatisfyingDaitaConstraints,
        .noRelaysSatisfyingObfuscationSettings,
        .noRelaysSatisfyingObfuscationPortConstraints,
        .noRelaysSatisfyingPortConstraints,
    ]

    private var shouldShowNotification: Bool = false

    var notificationDescriptor: InAppNotificationDescriptor? {
        guard shouldShowNotification else { return nil }
        if blockedStateReason != nil {
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
                        "Some of your settings have been migrated, please review the changes.",
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
        migratedSettingsUpdater: MigratedSettingsUpdater
    ) {
        self.tunnelManager = tunnelManager
        self.migratedSettingsUpdater = migratedSettingsUpdater
        super.init()
        addObservers()
    }

    // MARK: - Private methods
    private func addObservers() {
        let tunnelObserver = TunnelBlockObserver(
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

        let migratedSettingsObserver = MigratedSettingsObserverBlock { result in
            switch result {
            case .migrated:
                self.shouldShowNotification = true
            case .noChanges:
                self.shouldShowNotification = false
            }
            self.invalidate()
        }

        self.migratedSettingsObserverBlock = migratedSettingsObserver
        migratedSettingsUpdater.addObserver(migratedSettingsObserver)
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
                shouldShowNotification = false
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
                shouldShowNotification = false
                invalidate()
            }
        )
    }
}
