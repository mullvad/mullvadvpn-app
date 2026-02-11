//
//  NewAppStoreVersionInAppNotificationProvider.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-02-06.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadSettings
import MullvadTypes
import UIKit

final class NewAppStoreVersionInAppNotificationProvider:
    NotificationProvider, InAppNotificationProvider, @unchecked Sendable
{
    private var appStoreMetaDataService: AppStoreMetaDataService
    private var tunnelObserver: TunnelBlockObserver?

    private var canShowSafeNotification = false
    private var newAppStoreVersionAvailable = false
    private var tunnelIsSecured = false
    private var includeAllNetworksIsEnabled = false {
        didSet {
            if tunnelIsSecured && includeAllNetworksIsEnabled {
                canShowSafeNotification = true
            }
        }
    }

    init(tunnelManager: TunnelManager, appStoreMetaDataService: AppStoreMetaDataService) {
        self.appStoreMetaDataService = appStoreMetaDataService

        super.init()

        self.appStoreMetaDataService.onNewAppVersion = { [weak self] in
            guard let self else { return }
            invalidate(
                newAppStoreVersionAvailable: true,
                tunnelIsSecured: tunnelIsSecured,
                includeAllNetworksIsEnabled: includeAllNetworksIsEnabled
            )
        }

        let tunnelObserver = TunnelBlockObserver(
            didLoadConfiguration: { [weak self] tunnelManager in
                guard let self else { return }
                invalidate(
                    newAppStoreVersionAvailable: newAppStoreVersionAvailable,
                    tunnelIsSecured: tunnelManager.tunnelStatus.state.isSecured,
                    includeAllNetworksIsEnabled: tunnelManager.settings.includeAllNetworks
                )
            },
            didUpdateTunnelStatus: { [weak self] tunnelManager, _ in
                guard let self else { return }
                invalidate(
                    newAppStoreVersionAvailable: newAppStoreVersionAvailable,
                    tunnelIsSecured: tunnelManager.tunnelStatus.state.isSecured,
                    includeAllNetworksIsEnabled: tunnelManager.settings.includeAllNetworks
                )
            }
        )
        self.tunnelObserver = tunnelObserver

        tunnelManager.addObserver(tunnelObserver)
        self.appStoreMetaDataService.scheduleTimer()
    }

    override var identifier: NotificationProviderIdentifier {
        .newAppVersionInAppNotification
    }

    override var priority: NotificationPriority {
        .critical
    }

    // MARK: - InAppNotificationProvider

    var notificationDescriptor: InAppNotificationDescriptor? {
        let precondition = newAppStoreVersionAvailable && tunnelIsSecured
        guard precondition && (includeAllNetworksIsEnabled || canShowSafeNotification) else {
            return nil
        }

        let body: String
        if includeAllNetworksIsEnabled {
            body = NSLocalizedString(
                "“Force all apps” is enabled, please disable it or disconnect before updating or you will "
                    + "lose network connectivity.",
                comment: ""
            )
        } else {
            body = NSLocalizedString("Install the latest app version to stay up to date.", comment: "")
        }

        let string = NSMutableAttributedString(string: body)
        string.append(NSAttributedString(string: "\n"))
        string.append(
            NSAttributedString(
                string: "Tap here to go to AppStore",
                attributes: [
                    .font: UIFont.mullvadTinySemiBold,
                    .foregroundColor: UIColor.white,
                ]))

        return InAppNotificationDescriptor(
            identifier: identifier,
            style: .warning,
            title: NSLocalizedString("Update available", comment: ""),
            body: string,
            tapAction: InAppNotificationAction(
                handler: { [weak self] in
                    guard let self else { return }
                    NotificationManager.shared.notificationProvider(self, didReceiveAction: "\(identifier)")
                }
            )
        )
    }

    // MARK: - Private

    private func invalidate(
        newAppStoreVersionAvailable: Bool,
        tunnelIsSecured: Bool,
        includeAllNetworksIsEnabled: Bool
    ) {
        self.newAppStoreVersionAvailable = newAppStoreVersionAvailable
        self.tunnelIsSecured = tunnelIsSecured
        self.includeAllNetworksIsEnabled = includeAllNetworksIsEnabled

        invalidate()
    }
}
