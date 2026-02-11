//
//  NewAppVersionInAppNotificationProvider.swift
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

final class NewAppVersionInAppNotificationProvider:
    NotificationProvider, InAppNotificationProvider, @unchecked Sendable
{
    private var appVersionService: AppVersionService
    private var tunnelObserver: TunnelBlockObserver?

    private var hasEnabledIncludeAllNetworksAtLeastOnce = false
    private var newAppVersionIsAvailable = false
    private var tunnelIsSecured = false
    private var includeAllNetworksIsEnabled = false {
        didSet {
            if tunnelIsSecured && includeAllNetworksIsEnabled {
                hasEnabledIncludeAllNetworksAtLeastOnce = true
            }
        }
    }

    init(tunnelManager: TunnelManager, appVersionService: AppVersionService) {
        self.appVersionService = appVersionService

        super.init()

        self.appVersionService.onNewAppVersion = { [weak self] in
            guard let self else { return }

            newAppVersionIsAvailable = true
            invalidate()
        }

        let tunnelObserver = TunnelBlockObserver(
            didLoadConfiguration: { [weak self] tunnelManager in
                guard let self else { return }

                tunnelIsSecured = tunnelManager.tunnelStatus.state.isSecured
                includeAllNetworksIsEnabled = tunnelManager.settings.includeAllNetworks.includeAllNetworksIsEnabled
                invalidate()
            },
            didUpdateTunnelStatus: { [weak self] tunnelManager, _ in
                guard let self else { return }

                tunnelIsSecured = tunnelManager.tunnelStatus.state.isSecured
                includeAllNetworksIsEnabled = tunnelManager.settings.includeAllNetworks.includeAllNetworksIsEnabled
                invalidate()
            }
        )
        self.tunnelObserver = tunnelObserver

        tunnelManager.addObserver(tunnelObserver)
        self.appVersionService.scheduleTimer()
    }

    override var identifier: NotificationProviderIdentifier {
        .newAppVersionInAppNotification
    }

    override var priority: NotificationPriority {
        .critical
    }

    // MARK: - InAppNotificationProvider

    var notificationDescriptor: InAppNotificationDescriptor? {
        guard newAppVersionIsAvailable && tunnelIsSecured else {
            return nil
        }

        // The logic here is that if tunnel has been connected and IAN has been enabled at the same time at least once,
        // we should show the "safe" banner whenever the tunnel is up and IAN is disabled. We keep this state until
        // the app is restarted. If neither of the above is true, we shouldn't show a banner at all since we don't
        // want to urge people to update while we're using phased releases on AppStore.
        let body: String
        if includeAllNetworksIsEnabled {
            body = NSLocalizedString(
                String(
                    format:
                        "“%@” is enabled, please disable it or disconnect before updating or you will "
                        + "lose network connectivity.",
                    "Force all apps"
                ),
                comment: ""
            )
        } else if hasEnabledIncludeAllNetworksAtLeastOnce {
            body = NSLocalizedString("Install the latest app version to stay up to date.", comment: "")
        } else {
            return nil
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
            title: NSLocalizedString("Update available", comment: "").localizedUppercase,
            body: string,
            tapAction: InAppNotificationAction(
                handler: { [weak self] in
                    guard let self else { return }
                    NotificationManager.shared.notificationProvider(self, didReceiveAction: "\(identifier)")
                }
            )
        )
    }
}
