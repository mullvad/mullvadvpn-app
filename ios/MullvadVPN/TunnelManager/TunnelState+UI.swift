//
//  TunnelState+UI.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2024-05-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension TunnelState {
    enum TunnelControlActionButton {
        case connect
        case disconnect
        case cancel
    }

    var textColorForSecureLabel: UIColor {
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
        case let .connecting(_, isPostQuantum, _), let .reconnecting(_, isPostQuantum, _):
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

        case let .negotiatingEphemeralPeer(_, _, isPostQuantum, _):
            if isPostQuantum {
                NSLocalizedString(
                    "TUNNEL_STATE_NEGOTIATING_KEY",
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

        case let .connected(_, isPostQuantum, _):
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
                    value: "Connected",
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

    var localizedTitleForSelectLocationButton: String {
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
        case let .connecting(_, isPostQuantum, _):
            secureConnectionLabel(isPostQuantum: isPostQuantum)

        case let .negotiatingEphemeralPeer(_, _, isPostQuantum, _):
            secureConnectionLabel(isPostQuantum: isPostQuantum)

        case let .connected(tunnelInfo, isPostQuantum, _):
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

    var actionButton: TunnelControlActionButton {
        switch self {
        case .disconnected, .disconnecting(.nothing), .waitingForConnectivity(.noNetwork):
            .connect
        case .connecting, .pendingReconnect, .disconnecting(.reconnect), .waitingForConnectivity(.noConnection):
            .cancel
        case .negotiatingEphemeralPeer:
            .cancel
        case .connected, .reconnecting, .error:
            .disconnect
        }
    }

    var titleForCountryAndCity: String? {
        guard isSecured, let tunnelRelays = relays else {
            return nil
        }

        return "\(tunnelRelays.exit.location.country), \(tunnelRelays.exit.location.city)"
    }

    func titleForServer(daitaEnabled: Bool) -> String? {
        guard isSecured, let tunnelRelays = relays else {
            return nil
        }

        let exitName = tunnelRelays.exit.hostname
        let entryName = tunnelRelays.entry?.hostname
        let usingDaita = daitaEnabled == true

        return if let entryName {
            String(format: NSLocalizedString(
                "CONNECT_PANEL_TITLE",
                tableName: "Main",
                value: "%@ via %@\(usingDaita ? " using DAITA" : "")",
                comment: ""
            ), exitName, entryName)
        } else {
            String(format: NSLocalizedString(
                "CONNECT_PANEL_TITLE",
                tableName: "Main",
                value: "%@\(usingDaita ? " using DAITA" : "")",
                comment: ""
            ), exitName)
        }
    }

    func secureConnectionLabel(isPostQuantum: Bool) -> String {
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
    }
}
