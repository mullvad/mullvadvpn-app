//
//  TunnelErrorNotificationProvider.swift
//  TunnelErrorNotificationProvider
//
//  Created by pronebird on 20/08/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

class TunnelErrorNotificationProvider: NotificationProvider, InAppNotificationProvider, TunnelObserver {
    override var identifier: String {
        return "net.mullvad.MullvadVPN.TunnelErrorNotificationProvider"
    }

    var notificationDescriptor: InAppNotificationDescriptor? {
        guard let lastError = lastError else { return nil }

        return InAppNotificationDescriptor(
            identifier: identifier,
            style: .error,
            title: NSLocalizedString(
                "TUNNEL_ERROR_INAPP_NOTIFICATION_TITLE",
                value: "Tunnel error",
                comment: ""
            ),
            body: lastError.errorChainDescription ?? "No error description provided."
        )
    }

    private var lastError: TunnelManager.Error?

    override init() {
        super.init()

        TunnelManager.shared.addObserver(self)
    }

    // MARK: - TunnelObserver

    func tunnelManagerDidLoadConfiguration(_ manager: TunnelManager) {
        // no-op
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelState tunnelState: TunnelState) {
        // Reset error with each new connection attempt
        if case .connecting = tunnelState {
            lastError = nil
        }

        // Tell manager to refresh displayed notifications
        invalidate()
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelSettings tunnelSettings: TunnelSettingsV2?) {
        // no-op
    }

    func tunnelManager(_ manager: TunnelManager, didFailWithError error: TunnelManager.Error) {
        // Save tunnel error
        lastError = error

        // Tell manager to refresh displayed notifications
        invalidate()
    }


}
