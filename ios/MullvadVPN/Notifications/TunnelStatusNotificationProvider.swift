//
//  TunnelStatusNotificationProvider.swift
//  TunnelStatusNotificationProvider
//
//  Created by pronebird on 20/08/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

class TunnelStatusNotificationProvider: NotificationProvider, InAppNotificationProvider,
    TunnelObserver
{
    private var isWaitingForConnectivity = false
    private var packetTunnelError: String?
    private var tunnelManagerError: Error?

    override var identifier: String {
        return "net.mullvad.MullvadVPN.TunnelStatusNotificationProvider"
    }

    var notificationDescriptor: InAppNotificationDescriptor? {
        if let packetTunnelError = packetTunnelError {
            return notificationDescription(for: packetTunnelError)
        } else if let tunnelManagerError = tunnelManagerError {
            return notificationDescription(for: tunnelManagerError)
        } else if isWaitingForConnectivity {
            return connectivityNotificationDescription()
        } else {
            return nil
        }
    }

    init(tunnelManager: TunnelManager) {
        super.init()

        tunnelManager.addObserver(self)
        handleTunnelStatus(tunnelManager.tunnelStatus)
    }

    // MARK: - Private

    private func handleTunnelStatus(_ tunnelStatus: TunnelStatus) {
        let invalidateForTunnelError = updateLastTunnelError(
            tunnelStatus.packetTunnelStatus
                .lastError
        )
        let invalidateForManagerError = updateTunnelManagerError(tunnelStatus.state)
        let invalidateForConnectivity = updateConnectivity(tunnelStatus.state)

        if invalidateForTunnelError || invalidateForManagerError || invalidateForConnectivity {
            invalidate()
        }
    }

    private func updateLastTunnelError(_ lastTunnelError: String?) -> Bool {
        if packetTunnelError != lastTunnelError {
            packetTunnelError = lastTunnelError

            return true
        }

        return false
    }

    private func updateConnectivity(_ tunnelState: TunnelState) -> Bool {
        let isWaitingState = tunnelState == .waitingForConnectivity

        if isWaitingForConnectivity != isWaitingState {
            isWaitingForConnectivity = isWaitingState
            return true
        }

        return false
    }

    private func updateTunnelManagerError(_ tunnelState: TunnelState) -> Bool {
        switch tunnelState {
        case .connecting, .connected, .reconnecting:
            // As of now, tunnel manager error can be received only when starting or stopping
            // the tunnel. Make sure to reset it on each connection attempt.
            if tunnelManagerError != nil {
                tunnelManagerError = nil
                return true
            }

        default:
            break
        }

        return false
    }

    private func notificationDescription(for packetTunnelError: String)
        -> InAppNotificationDescriptor
    {
        return InAppNotificationDescriptor(
            identifier: identifier,
            style: .error,
            title: NSLocalizedString(
                "TUNNEL_LEAKING_INAPP_NOTIFICATION_TITLE",
                value: "NETWORK TRAFFIC MIGHT BE LEAKING",
                comment: ""
            ),
            body: String(
                format: NSLocalizedString(
                    "PACKET_TUNNEL_ERROR_INAPP_NOTIFICATION_BODY",
                    value: "Could not configure VPN: %@",
                    comment: ""
                ),
                packetTunnelError
            )
        )
    }

    private func notificationDescription(for error: Error) -> InAppNotificationDescriptor {
        let body: String

        if let startError = error as? StartTunnelError {
            body = String(
                format: NSLocalizedString(
                    "START_TUNNEL_ERROR_INAPP_NOTIFICATION_BODY",
                    value: "Failed to start the tunnel: %@.",
                    comment: ""
                ),
                startError.underlyingError?.localizedDescription ?? ""
            )
        } else if let stopError = error as? StopTunnelError {
            body = String(
                format: NSLocalizedString(
                    "STOP_TUNNEL_ERROR_INAPP_NOTIFICATION_BODY",
                    value: "Failed to stop the tunnel: %@.",
                    comment: ""
                ),
                stopError.underlyingError?.localizedDescription ?? ""
            )
        } else {
            body = error.localizedDescription
        }

        return InAppNotificationDescriptor(
            identifier: identifier,
            style: .error,
            title: NSLocalizedString(
                "TUNNEL_MANAGER_ERROR_INAPP_NOTIFICATION_TITLE",
                value: "TUNNEL ERROR",
                comment: ""
            ),
            body: body
        )
    }

    private func connectivityNotificationDescription() -> InAppNotificationDescriptor {
        return InAppNotificationDescriptor(
            identifier: identifier,
            style: .warning,
            title: NSLocalizedString(
                "TUNNEL_NO_CONNECTIVITY_INAPP_NOTIFICATION_TITLE",
                value: "NETWORK ISSUES",
                comment: ""
            ),
            body: NSLocalizedString(
                "TUNNEL_NO_CONNECTIVITY_INAPP_NOTIFICATION_BODY",
                value: """
                Your device is offline. The tunnel will automatically connect once \
                your device is back online.
                """,
                comment: ""
            )
        )
    }

    // MARK: - TunnelObserver

    func tunnelManagerDidLoadConfiguration(_ manager: TunnelManager) {
        // no-op
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelStatus tunnelStatus: TunnelStatus) {
        handleTunnelStatus(tunnelStatus)
    }

    func tunnelManager(
        _ manager: TunnelManager,
        didUpdateTunnelSettings tunnelSettings: TunnelSettingsV2
    ) {
        // no-op
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateDeviceState deviceState: DeviceState) {
        // no-op
    }

    func tunnelManager(_ manager: TunnelManager, didFailWithError error: Error) {
        tunnelManagerError = error
    }
}
