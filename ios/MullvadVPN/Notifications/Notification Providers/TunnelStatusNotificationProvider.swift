//
//  TunnelStatusNotificationProvider.swift
//  TunnelStatusNotificationProvider
//
//  Created by pronebird on 20/08/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import PacketTunnelCore

final class TunnelStatusNotificationProvider: NotificationProvider, InAppNotificationProvider {
    private var isWaitingForConnectivity = false
    private var noNetwork = false
    private var packetTunnelError: BlockedStateReason?
    private var tunnelManagerError: Error?
    private var tunnelObserver: TunnelBlockObserver?

    override var identifier: NotificationProviderIdentifier {
        .tunnelStatusNotificationProvider
    }

    var notificationDescriptor: InAppNotificationDescriptor? {
        if let packetTunnelError {
            return notificationDescription(for: packetTunnelError)
        } else if let tunnelManagerError {
            return notificationDescription(for: tunnelManagerError)
        } else if isWaitingForConnectivity {
            return connectivityNotificationDescription()
        } else if noNetwork {
            return noNetworkNotificationDescription()
        } else {
            return nil
        }
    }

    init(tunnelManager: TunnelManager) {
        super.init()

        let tunnelObserver = TunnelBlockObserver(
            didLoadConfiguration: { [weak self] tunnelManager in
                self?.handleTunnelStatus(tunnelManager.tunnelStatus)
            },
            didUpdateTunnelStatus: { [weak self] _, tunnelStatus in
                self?.handleTunnelStatus(tunnelStatus)
            },
            didFailWithError: { [weak self] _, error in
                self?.tunnelManagerError = error
            }
        )
        self.tunnelObserver = tunnelObserver

        tunnelManager.addObserver(tunnelObserver)
    }

    // MARK: - Private

    private func handleTunnelStatus(_ tunnelStatus: TunnelStatus) {
        let invalidateForTunnelError = updateLastTunnelError(tunnelStatus.state)
        let invalidateForManagerError = updateTunnelManagerError(tunnelStatus.state)
        let invalidateForConnectivity = updateConnectivity(tunnelStatus.state)
        let invalidateForNetwork = updateNetwork(tunnelStatus.state)

        if invalidateForTunnelError || invalidateForManagerError || invalidateForConnectivity || invalidateForNetwork {
            invalidate()
        }
    }

    private func updateLastTunnelError(_ tunnelState: TunnelState) -> Bool {
        let lastTunnelError = tunnelError(from: tunnelState)

        if packetTunnelError != lastTunnelError {
            packetTunnelError = lastTunnelError

            return true
        }

        return false
    }

    private func updateConnectivity(_ tunnelState: TunnelState) -> Bool {
        let isWaitingState = tunnelState == .waitingForConnectivity(.noConnection)

        if isWaitingForConnectivity != isWaitingState {
            isWaitingForConnectivity = isWaitingState
            return true
        }

        return false
    }

    private func updateNetwork(_ tunnelState: TunnelState) -> Bool {
        let isWaitingState = tunnelState == .waitingForConnectivity(.noNetwork)

        if noNetwork != isWaitingState {
            noNetwork = isWaitingState
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

    private func tunnelError(from tunnelState: TunnelState) -> BlockedStateReason? {
        if case let .error(blockedStateReason) = tunnelState,
           ![.accountExpired, .deviceRevoked].contains(blockedStateReason) {
            return blockedStateReason
        }

        return nil
    }

    private func notificationDescription(for packetTunnelError: BlockedStateReason) -> InAppNotificationDescriptor {
        InAppNotificationDescriptor(
            identifier: identifier,
            style: .error,
            title: NSLocalizedString(
                "TUNNEL_BLOCKED_INAPP_NOTIFICATION_TITLE",
                value: "BLOCKING INTERNET",
                comment: ""
            ),
            body: .init(string: String(
                format: NSLocalizedString(
                    "TUNNEL_BLOCKED_INAPP_NOTIFICATION_BODY",
                    value: localizedReasonForBlockedStateError(packetTunnelError),
                    comment: ""
                )
            ))
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
            body: .init(string: body)
        )
    }

    private func connectivityNotificationDescription() -> InAppNotificationDescriptor {
        InAppNotificationDescriptor(
            identifier: identifier,
            style: .warning,
            title: NSLocalizedString(
                "TUNNEL_NO_CONNECTIVITY_INAPP_NOTIFICATION_TITLE",
                value: "NETWORK ISSUES",
                comment: ""
            ),
            body: .init(
                string: NSLocalizedString(
                    "TUNNEL_NO_CONNECTIVITY_INAPP_NOTIFICATION_BODY",
                    value: """
                    Your device is offline. The tunnel will automatically connect once \
                    your device is back online.
                    """,
                    comment: ""
                )
            )
        )
    }

    private func noNetworkNotificationDescription() -> InAppNotificationDescriptor {
        InAppNotificationDescriptor(
            identifier: identifier,
            style: .warning,
            title: NSLocalizedString(
                "TUNNEL_NO_NETWORK_INAPP_NOTIFICATION_TITLE",
                value: "NETWORK ISSUES",
                comment: ""
            ),
            body: .init(
                string: NSLocalizedString(
                    "TUNNEL_NO_NETWORK_INAPP_NOTIFICATION_BODY",
                    value: """
                    Your device is offline. Try connecting again when the device \
                    has access to Internet.
                    """,
                    comment: ""
                )
            )
        )
    }

    private func localizedReasonForBlockedStateError(_ error: BlockedStateReason) -> String {
        let errorString: String

        switch error {
        case .outdatedSchema:
            errorString = "Unable to start tunnel connection after update. Please disconnect and reconnect."
        case .noRelaysSatisfyingConstraints:
            errorString = "No servers match your settings, try changing server or other settings."
        case .invalidAccount:
            errorString = "You are logged in with an invalid account number. Please log out and try another one."
        case .deviceLoggedOut:
            errorString = "Unable to authenticate account. Please log out and log back in."
        default:
            errorString = "Unable to start tunnel connection. Please send a problem report."
        }

        return NSLocalizedString(
            "BLOCKED_STATE_ERROR_TITLE",
            tableName: "Main",
            value: errorString,
            comment: ""
        )
    }
}
