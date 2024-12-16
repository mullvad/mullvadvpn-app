//
//  ConnectionViewViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-09.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Combine
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

    @Published var tunnelStatus: TunnelStatus
    @Published var showsActivityIndicator = false

    var combinedState: Publishers.CombineLatest<
        Published<TunnelStatus>.Publisher,
        Published<Bool>.Publisher
    > {
        $tunnelStatus.combineLatest($showsActivityIndicator)
    }

    var isConnected: Bool {
        tunnelStatus.state != .disconnected
    }

    init(tunnelStatus: TunnelStatus) {
        self.tunnelStatus = tunnelStatus
    }
}

extension ConnectionViewViewModel {
    var textColorForSecureLabel: UIColor {
        switch tunnelStatus.state {
        case .connecting, .reconnecting, .waitingForConnectivity(.noConnection), .negotiatingEphemeralPeer:
            .white
        case .connected:
            .successColor
        case .disconnecting, .disconnected, .pendingReconnect, .waitingForConnectivity(.noNetwork), .error:
            .dangerColor
        }
    }

    var disableButtons: Bool {
        if case .waitingForConnectivity(.noNetwork) = tunnelStatus.state {
            return true
        }

        return false
    }

    var localizedTitleForSecureLabel: LocalizedStringKey {
        switch tunnelStatus.state {
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
        switch tunnelStatus.state {
        case .disconnecting, .pendingReconnect, .disconnected:
            LocalizedStringKey("Select location")
        case .connecting, .connected, .reconnecting, .waitingForConnectivity, .negotiatingEphemeralPeer, .error:
            LocalizedStringKey("Switch location")
        }
    }

    var localizedAccessibilityLabel: LocalizedStringKey {
        switch tunnelStatus.state {
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
        switch tunnelStatus.state {
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
        guard let tunnelRelays = tunnelStatus.state.relays else {
            return nil
        }

        return LocalizedStringKey("\(tunnelRelays.exit.location.country), \(tunnelRelays.exit.location.city)")
    }

    var titleForServer: LocalizedStringKey? {
        guard let tunnelRelays = tunnelStatus.state.relays else {
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

    var inAddress: LocalizedStringKey? {
        guard let tunnelRelays = tunnelStatus.state.relays else {
            return nil
        }

        let observedTunnelState = tunnelStatus.observedState

        var portAndTransport = ""
        if let inPort = observedTunnelState.connectionState?.remotePort {
            let protocolLayer = observedTunnelState.connectionState?.transportLayer == .tcp ? "TCP" : "UDP"
            portAndTransport = ":\(inPort) \(protocolLayer)"
        }

        guard
            let address = tunnelRelays.entry?.endpoint.ipv4Relay.description
                ?? tunnelStatus.state.relays?.exit.endpoint.ipv4Relay.description
        else {
            return nil
        }

        return LocalizedStringKey("\(address)\(portAndTransport)")
    }

    var outAddressIpv4: LocalizedStringKey? {
        guard let address = tunnelStatus.state.relays?.exit.endpoint.ipv4Relay.description else {
            return nil
        }

        return LocalizedStringKey("\(address)")
    }

    var outAddressIpv6: LocalizedStringKey? {
        guard let address = tunnelStatus.state.relays?.exit.endpoint.ipv6Relay?.description else {
            return nil
        }

        return LocalizedStringKey("\(address)")
    }
}
