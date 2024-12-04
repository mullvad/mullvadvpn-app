//
//  ConnectionViewViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-09.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

class ConnectionViewViewModel: ObservableObject {
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

    @Published var tunnelState: TunnelState
    @Published var showsActivityIndicator = false

    init(tunnelState: TunnelState) {
        self.tunnelState = tunnelState
    }
}

extension ConnectionViewViewModel {
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

    var localizedTitleForSecureLabel: LocalizedStringKey {
        switch tunnelState {
        case .connecting, .reconnecting, .negotiatingEphemeralPeer:
            LocalizedStringKey("Connecting")
        case .connected:
            LocalizedStringKey("Connected")
        case .disconnecting(.nothing):
            LocalizedStringKey("Disconnecting")
        case .disconnecting(.reconnect), .pendingReconnect:
            LocalizedStringKey("Reconnecting")
        case .disconnected:
            LocalizedStringKey("Disconnected")
        case .waitingForConnectivity(.noConnection), .error:
            LocalizedStringKey("Blocked connection")
        case .waitingForConnectivity(.noNetwork):
            LocalizedStringKey("No network")
        }
    }

    var localizedTitleForSelectLocationButton: LocalizedStringKey {
        switch tunnelState {
        case .disconnecting, .pendingReconnect, .disconnected:
            LocalizedStringKey("Select location")
        case .connecting, .connected, .reconnecting, .waitingForConnectivity, .negotiatingEphemeralPeer, .error:
            LocalizedStringKey("Switch location")
        }
    }

    var localizedAccessibilityLabel: LocalizedStringKey {
        switch tunnelState {
        case .disconnected, .waitingForConnectivity, .disconnecting, .pendingReconnect, .error:
            localizedTitleForSecureLabel
        case let .connected(tunnelInfo, _, _):
            LocalizedStringKey("Connected to \(tunnelInfo.exit.location.city), \(tunnelInfo.exit.location.country)")
        case let .connecting(tunnelInfo, _, _):
            if let tunnelInfo {
                LocalizedStringKey(
                    "Connecting to \(tunnelInfo.exit.location.city), \(tunnelInfo.exit.location.country)"
                )
            } else {
                localizedTitleForSecureLabel
            }
        case let .reconnecting(tunnelInfo, _, _), let .negotiatingEphemeralPeer(tunnelInfo, _, _, _):
            LocalizedStringKey("Reconnecting to \(tunnelInfo.exit.location.city), \(tunnelInfo.exit.location.country)")
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

    var titleForCountryAndCity: LocalizedStringKey? {
        guard tunnelState.isSecured, let tunnelRelays = tunnelState.relays else {
            return nil
        }

        return LocalizedStringKey("\(tunnelRelays.exit.location.country), \(tunnelRelays.exit.location.city)")
    }

    var titleForServer: LocalizedStringKey? {
        guard tunnelState.isSecured, let tunnelRelays = tunnelState.relays else {
            return nil
        }

        let exitName = tunnelRelays.exit.hostname
        let entryName = tunnelRelays.entry?.hostname

        return if let entryName {
            LocalizedStringKey("\(exitName) via \(entryName)")
        } else {
            LocalizedStringKey("\(exitName)")
        }
    }
}

extension ConnectionViewViewModel {
    @ViewBuilder
    func locationButton(with action: ButtonAction?) -> some View {
        switch tunnelState {
        case .connecting, .connected, .reconnecting, .waitingForConnectivity, .negotiatingEphemeralPeer, .error:
            SplitMainButton(
                text: localizedTitleForSelectLocationButton,
                image: .iconReload,
                style: .default,
                disabled: disableButtons,
                primaryAction: { action?(.selectLocation) },
                secondaryAction: { action?(.reconnect) }
            )
        case .disconnecting, .pendingReconnect, .disconnected:
            MainButton(
                text: localizedTitleForSelectLocationButton,
                style: .default,
                disabled: disableButtons,
                action: { action?(.selectLocation) }
            )
        }
    }

    @ViewBuilder
    func actionButton(with action: ButtonAction?) -> some View {
        switch actionButton {
        case .connect:
            MainButton(
                text: LocalizedStringKey("Connect"),
                style: .success,
                disabled: disableButtons,
                action: { action?(.connect) }
            )
        case .disconnect:
            MainButton(
                text: LocalizedStringKey("Disconnect"),
                style: .danger,
                disabled: disableButtons,
                action: { action?(.disconnect) }
            )
        case .cancel:
            MainButton(
                text: LocalizedStringKey(
                    tunnelState == .waitingForConnectivity(.noConnection)
                        ? "Disconnect"
                        : "Cancel"
                ),
                style: .danger,
                disabled: disableButtons,
                action: { action?(.cancel) }
            )
        }
    }
}
