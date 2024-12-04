//
//  ConnectionViewViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-09.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct ConnectionViewViewModel {
    enum TunnelControlActionButton {
        case connect
        case disconnect
        case cancel
    }

    enum TunnelControlAction {
        case connect
        case disconnect
        case cancel
        case reconnect
        case selectLocation
    }

    @State var tunnelState: TunnelState

    var textColorForSecureLabel: UIColor {
        switch tunnelState {
        case .connecting, .reconnecting, .waitingForConnectivity(.noConnection), .negotiatingEphemeralPeer:
            .white
        case .connected:
            .successColor
        case .disconnecting, .disconnected, .pendingReconnect, .waitingForConnectivity(.noNetwork), .error:
            .dangerColor
        }
    }

    var disableButtons: Bool {
        if case .waitingForConnectivity(.noNetwork) = tunnelState {
            return true
        }

        return false
    }

    var localizedTitleForSecureLabel: String {
        switch tunnelState {
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

    var localizedTitleForSelectLocationButton: String {
        switch tunnelState {
        case .disconnecting, .pendingReconnect, .disconnected:
            NSLocalizedString(
                "SWITCH_LOCATION_BUTTON_TITLE",
                tableName: "Main",
                value: "Select location",
                comment: ""
            )

        case .connecting, .connected, .reconnecting, .waitingForConnectivity, .negotiatingEphemeralPeer, .error:
            NSLocalizedString(
                "SWITCH_LOCATION_BUTTON_TITLE",
                tableName: "Main",
                value: "Switch location",
                comment: ""
            )
        }
    }

    var localizedAccessibilityLabel: String {
        switch tunnelState {
        case .disconnected, .waitingForConnectivity, .disconnecting, .pendingReconnect, .error:
            localizedTitleForSecureLabel

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

        case let .connecting(tunnelInfo, _, _):
            if let tunnelInfo {
                String(
                    format: NSLocalizedString(
                        "TUNNEL_STATE_RECONNECTING_ACCESSIBILITY_LABEL",
                        tableName: "Main",
                        value: "Connecting to %@, %@",
                        comment: ""
                    ),
                    tunnelInfo.exit.location.city,
                    tunnelInfo.exit.location.country
                )
            } else {
                localizedTitleForSecureLabel
            }

        case let .reconnecting(tunnelInfo, _, _), let .negotiatingEphemeralPeer(tunnelInfo, _, _, _):
            String(
                format: NSLocalizedString(
                    "TUNNEL_STATE_RECONNECTING_ACCESSIBILITY_LABEL",
                    tableName: "Main",
                    value: "Connecting to %@, %@",
                    comment: ""
                ),
                tunnelInfo.exit.location.city,
                tunnelInfo.exit.location.country
            )
        }
    }

    var actionButton: TunnelControlActionButton {
        switch tunnelState {
        case .disconnected, .disconnecting(.nothing), .waitingForConnectivity(.noNetwork):
            .connect
        case .connecting, .pendingReconnect, .disconnecting(.reconnect), .waitingForConnectivity(.noConnection),
             .negotiatingEphemeralPeer:
            .cancel
        case .connected, .reconnecting, .error:
            .disconnect
        }
    }

    var titleForCountryAndCity: String? {
        guard tunnelState.isSecured, let tunnelRelays = tunnelState.relays else {
            return nil
        }

        return "\(tunnelRelays.exit.location.country), \(tunnelRelays.exit.location.city)"
    }

    var titleForServer: String? {
        guard tunnelState.isSecured, let tunnelRelays = tunnelState.relays else {
            return nil
        }

        let exitName = tunnelRelays.exit.hostname
        let entryName = tunnelRelays.entry?.hostname

        return if let entryName {
            String(format: NSLocalizedString(
                "CONNECT_PANEL_TITLE",
                tableName: "Main",
                value: "%@ via %@",
                comment: ""
            ), exitName, entryName)
        } else {
            String(format: NSLocalizedString(
                "CONNECT_PANEL_TITLE",
                tableName: "Main",
                value: "%@",
                comment: ""
            ), exitName)
        }
    }
}

extension ConnectionViewViewModel {
    @ViewBuilder
    func locationButton(with action: ButtonAction?) -> some View {
        switch tunnelState {
        case .connecting, .connected, .reconnecting, .waitingForConnectivity, .negotiatingEphemeralPeer, .error:
            MainButton(
                text: localizedTitleForSelectLocationButton,
                style: .default,
                disabled: disableButtons,
                action: { action?(.selectLocation) }
            )
        case .disconnecting, .pendingReconnect, .disconnected:
            SplitMainButton(
                text: localizedTitleForSelectLocationButton,
                image: .iconReload,
                style: .default,
                disabled: disableButtons,
                primaryAction: { action?(.selectLocation) },
                secondaryAction: { action?(.reconnect) }
            )
        }
    }

    @ViewBuilder
    func actionButton(with action: ButtonAction?) -> some View {
        switch actionButton {
        case .connect:
            MainButton(
                text: NSLocalizedString(
                    "CONNECT_BUTTON_TITLE",
                    tableName: "Main",
                    value: "Connect",
                    comment: ""
                ),
                style: .success,
                disabled: disableButtons,
                action: { action?(.connect) }
            )
        case .disconnect:
            MainButton(
                text: NSLocalizedString(
                    "DISCONNECT_BUTTON_TITLE",
                    tableName: "Main",
                    value: "Disconnect",
                    comment: ""
                ),
                style: .danger,
                disabled: disableButtons,
                action: { action?(.disconnect) }
            )
        case .cancel:
            MainButton(
                text: NSLocalizedString(
                    "CANCEL_BUTTON_TITLE",
                    tableName: "Main",
                    value: tunnelState == .waitingForConnectivity(.noConnection) ? "Disconnect" : "Cancel",
                    comment: ""
                ),
                style: .danger,
                disabled: disableButtons,
                action: { action?(.cancel) }
            )
        }
    }
}
