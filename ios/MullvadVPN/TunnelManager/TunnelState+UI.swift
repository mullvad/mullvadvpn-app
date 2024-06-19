//
//  TunnelState+UI.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2024-05-03.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension TunnelState {
    var textColorForSecureLabel: UIColor {
        switch self {
        case .connecting, .reconnecting, .waitingForConnectivity(.noConnection), .negotiatingPostQuantumKey:
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
        case let .connecting(_, isPostQuantum), let .reconnecting(_, isPostQuantum):
            if isPostQuantum {
                NSLocalizedString(
                    "TUNNEL_STATE_PQ_CONNECTING",
                    tableName: "Main",
                    value: "Creating quantum secure connection",
                    comment: ""
                )
            } else {
                NSLocalizedString(
                    "TUNNEL_STATE_CONNECTING",
                    tableName: "Main",
                    value: "Creating secure connection",
                    comment: ""
                )
            }

        case .negotiatingPostQuantumKey:
            NSLocalizedString(
                "TUNNEL_STATE_NEGOTIATING_KEY",
                tableName: "Main",
                value: "Creating quantum secure connection",
                comment: ""
            )

        case let .connected(_, isPostQuantum):
            if isPostQuantum {
                NSLocalizedString(
                    "TUNNEL_STATE_PQ_CONNECTED",
                    tableName: "Main",
                    value: "Quantum secure connection",
                    comment: ""
                )
            } else {
                NSLocalizedString(
                    "TUNNEL_STATE_CONNECTED",
                    tableName: "Main",
                    value: "Secure connection",
                    comment: ""
                )
            }

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
                value: "Unsecured connection",
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

        case .negotiatingPostQuantumKey:
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
        case let .connecting(_, isPostQuantum):
            if isPostQuantum {
                NSLocalizedString(
                    "TUNNEL_STATE_PQ_CONNECTING_ACCESSIBILITY_LABEL",
                    tableName: "Main",
                    value: "Creating quantum secure connection",
                    comment: ""
                )
            } else {
                NSLocalizedString(
                    "TUNNEL_STATE_CONNECTING_ACCESSIBILITY_LABEL",
                    tableName: "Main",
                    value: "Creating secure connection",
                    comment: ""
                )
            }

        case .negotiatingPostQuantumKey:
            NSLocalizedString(
                "TUNNEL_STATE_CONNECTING_ACCESSIBILITY_LABEL",
                tableName: "Main",
                value: "Creating quantum secure connection",
                comment: ""
            )

        case let .connected(tunnelInfo, isPostQuantum):
            if isPostQuantum {
                String(
                    format: NSLocalizedString(
                        "TUNNEL_STATE_PQ_CONNECTED_ACCESSIBILITY_LABEL",
                        tableName: "Main",
                        value: "Quantum secure connection. Connected to %@, %@",
                        comment: ""
                    ),
                    tunnelInfo.exit.location.city,
                    tunnelInfo.exit.location.country
                )
            } else {
                String(
                    format: NSLocalizedString(
                        "TUNNEL_STATE_CONNECTED_ACCESSIBILITY_LABEL",
                        tableName: "Main",
                        value: "Secure connection. Connected to %@, %@",
                        comment: ""
                    ),
                    tunnelInfo.exit.location.city,
                    tunnelInfo.exit.location.country
                )
            }

        case .disconnected:
            NSLocalizedString(
                "TUNNEL_STATE_DISCONNECTED_ACCESSIBILITY_LABEL",
                tableName: "Main",
                value: "Unsecured connection",
                comment: ""
            )

        case let .reconnecting(tunnelInfo, _):
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
