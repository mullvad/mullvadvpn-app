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

        let body = (lastError as? LocalizedNotificationError)?.localizedNotificationBody
            ?? lastError.localizedDescription

        return InAppNotificationDescriptor(
            identifier: identifier,
            style: .error,
            title: NSLocalizedString(
                "TUNNEL_ERROR_INAPP_NOTIFICATION_TITLE",
                value: "TUNNEL ERROR",
                comment: ""
            ),
            body: body
        )
    }

    private var lastError: Error?

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

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelSettings tunnelSettings: TunnelSettingsV2) {
        // no-op
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateDeviceState deviceState: DeviceState) {
        // no-op
    }

    func tunnelManager(_ manager: TunnelManager, didFailWithError error: Error) {
        // Save tunnel error
        lastError = error

        // Tell manager to refresh displayed notifications
        invalidate()
    }
}

protocol LocalizedNotificationError {
    var localizedNotificationBody: String? { get }
}

extension StartTunnelError: LocalizedNotificationError {
    var localizedNotificationBody: String? {
        return String(
            format: NSLocalizedString(
                "START_TUNNEL_ERROR_INAPP_NOTIFICATION_BODY",
                value: "Failed to start the tunnel: %@.",
                comment: ""
            ),
            underlyingError.localizedDescription
        )
    }
}

extension StopTunnelError: LocalizedNotificationError {
    var localizedNotificationBody: String? {
        return String(
            format: NSLocalizedString(
                "STOP_TUNNEL_ERROR_INAPP_NOTIFICATION_BODY",
                value: "Failed to stop the tunnel: %@.",
                comment: ""
            ),
            underlyingError.localizedDescription
        )
    }
}
