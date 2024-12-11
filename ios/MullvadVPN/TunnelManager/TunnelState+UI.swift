//
//  TunnelState+UI.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2024-05-03.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension TunnelState {
    var textColorForConnectionStatusLabel: UIColor {
        switch self {
        case .connecting, .reconnecting, .waitingForConnectivity(.noConnection), .negotiatingEphemeralPeer:
            .white

        case .connected:
            .successColor

        case .disconnecting, .disconnected, .pendingReconnect, .waitingForConnectivity(.noNetwork), .error:
            .dangerColor
        }
    }

    var shouldEnableButtons: Bool {
        if case .waitingForConnectivity(.noNetwork) = self {
            return false
        }

        return true
    }

    var localizedTitleForSecureLabel: String {
        switch self {
        case .connecting, .reconnecting, .negotiatingEphemeralPeer:
            NSLocalizedString(
                "TUNNEL_STATE_CONNECTING",
                tableName: "Main",
                value: "Connecting...",
                comment: ""
            )
        case .connected:
            NSLocalizedString(
                "TUNNEL_STATE_CONNECTED",
                tableName: "Main",
                value: "Connected",
                comment: ""
            )

        case .disconnecting(.nothing):
            NSLocalizedString(
                "TUNNEL_STATE_DISCONNECTING",
                tableName: "Main",
                value: "Disconnecting",
                comment: ""
            )
        case .disconnecting(.reconnect), .pendingReconnect:
            NSLocalizedString(
                "TUNNEL_STATE_PENDING_RECONNECT",
                tableName: "Main",
                value: "Reconnecting",
                comment: ""
            )

        case .disconnected:
            NSLocalizedString(
                "TUNNEL_STATE_DISCONNECTED",
                tableName: "Main",
                value: "Disconnected",
                comment: ""
            )

        case .waitingForConnectivity(.noConnection), .error:
            NSLocalizedString(
                "TUNNEL_STATE_WAITING_FOR_CONNECTIVITY",
                tableName: "Main",
                value: "Blocked connection",
                comment: ""
            )

        case .waitingForConnectivity(.noNetwork):
            NSLocalizedString(
                "TUNNEL_STATE_NO_NETWORK",
                tableName: "Main",
                value: "No network",
                comment: ""
            )
        }
    }

    var localizedTitleForSelectLocationButton: String? {
        switch self {
        case .disconnecting(.reconnect), .pendingReconnect:
            NSLocalizedString(
                "SWITCH_LOCATION_BUTTON_TITLE",
                tableName: "Main",
                value: "Select location",
                comment: ""
            )

        case .disconnected, .disconnecting(.nothing):
            NSLocalizedString(
                "SELECT_LOCATION_BUTTON_TITLE",
                tableName: "Main",
                value: "Select location",
                comment: ""
            )

        case .connecting, .connected, .reconnecting, .waitingForConnectivity, .error:
            NSLocalizedString(
                "SWITCH_LOCATION_BUTTON_TITLE",
                tableName: "Main",
                value: "Switch location",
                comment: ""
            )

        case .negotiatingEphemeralPeer:
            NSLocalizedString(
                "SWITCH_LOCATION_BUTTON_TITLE",
                tableName: "Main",
                value: "Switch location",
                comment: ""
            )
        }
    }

    var localizedAccessibilityLabel: String {
        switch self {
        case .connecting, .negotiatingEphemeralPeer:
            NSLocalizedString(
                "TUNNEL_STATE_CONNECTING_ACCESSIBILITY_LABEL",
                tableName: "Main",
                value: "Connecting",
                comment: ""
            )

        case let .connected(tunnelInfo, _, _):
            String(
                format: NSLocalizedString(
                    "TUNNEL_STATE_CONNECTED_ACCESSIBILITY_LABEL",
                    tableName: "Main",
                    value: "Connected to %@, %@",
                    comment: ""
                ),
                tunnelInfo.exit.location.city,
                tunnelInfo.exit.location.country
            )

        case .disconnected:
            NSLocalizedString(
                "TUNNEL_STATE_DISCONNECTED_ACCESSIBILITY_LABEL",
                tableName: "Main",
                value: "Disconnected",
                comment: ""
            )

        case let .reconnecting(tunnelInfo, _, _):
            String(
                format: NSLocalizedString(
                    "TUNNEL_STATE_RECONNECTING_ACCESSIBILITY_LABEL",
                    tableName: "Main",
                    value: "Reconnecting to %@, %@",
                    comment: ""
                ),
                tunnelInfo.exit.location.city,
                tunnelInfo.exit.location.country
            )

        case .waitingForConnectivity(.noConnection), .error:
            NSLocalizedString(
                "TUNNEL_STATE_WAITING_FOR_CONNECTIVITY_ACCESSIBILITY_LABEL",
                tableName: "Main",
                value: "Blocked connection",
                comment: ""
            )

        case .waitingForConnectivity(.noNetwork):
            NSLocalizedString(
                "TUNNEL_STATE_NO_NETWORK_ACCESSIBILITY_LABEL",
                tableName: "Main",
                value: "No network",
                comment: ""
            )

        case .disconnecting(.nothing):
            NSLocalizedString(
                "TUNNEL_STATE_DISCONNECTING_ACCESSIBILITY_LABEL",
                tableName: "Main",
                value: "Disconnecting",
                comment: ""
            )

        case .disconnecting(.reconnect), .pendingReconnect:
            NSLocalizedString(
                "TUNNEL_STATE_PENDING_RECONNECT_ACCESSIBILITY_LABEL",
                tableName: "Main",
                value: "Reconnecting",
                comment: ""
            )
        }
    }
}
