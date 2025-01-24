//
//  ConnectionViewViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-09.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadSettings
import MullvadTypes
import SwiftUI

class ConnectionViewViewModel: ObservableObject {
    enum TunnelActionButton {
        case connect
        case disconnect
        case cancel
    }

    enum TunnelAction {
        case connect
        case disconnect
        case cancel
        case reconnect
        case selectLocation
    }

    @Published private(set) var tunnelStatus: TunnelStatus
    @Published var outgoingConnectionInfo: OutgoingConnectionInfo?
    @Published var showsActivityIndicator = false

    let relayConstraints: RelayConstraints
    let customListRepository: CustomListRepositoryProtocol

    var combinedState: Publishers.CombineLatest<
        Published<TunnelStatus>.Publisher,
        Published<Bool>.Publisher
    > {
        $tunnelStatus.combineLatest($showsActivityIndicator)
    }

    var tunnelIsConnected: Bool {
        if case .connected = tunnelStatus.state {
            true
        } else {
            false
        }
    }

    var connectionName: String? {
        if case let .only(loc) = relayConstraints.exitLocations {
            loc.customListSelection.flatMap { customListRepository.fetch(by: $0.listId)?.name } ?? loc.locations.first?
                .stringRepresentation
        } else {
            "Somewhere"
        }
//        nil
    }

    init(
        tunnelStatus: TunnelStatus,
        relayConstraints: RelayConstraints,
        customListRepository: CustomListRepositoryProtocol
    ) {
        self.tunnelStatus = tunnelStatus
        self.relayConstraints = relayConstraints
        self.customListRepository = customListRepository
    }

    func update(tunnelStatus: TunnelStatus) {
        self.tunnelStatus = tunnelStatus

        if !tunnelIsConnected {
            outgoingConnectionInfo = nil
        }
    }
}

extension ConnectionViewViewModel {
    var showConnectionDetails: Bool {
        switch tunnelStatus.state {
        case .connecting, .reconnecting, .waitingForConnectivity(.noConnection), .negotiatingEphemeralPeer,
             .connected, .pendingReconnect:
            true
        case .disconnecting, .disconnected, .waitingForConnectivity(.noNetwork), .error:
            false
        }
    }

    var textColorForSecureLabel: UIColor {
        switch tunnelStatus.state {
        case .connecting, .reconnecting, .waitingForConnectivity(.noConnection), .negotiatingEphemeralPeer,
             .pendingReconnect, .disconnecting:
            .white
        case .connected:
            .successColor
        case .disconnected, .waitingForConnectivity(.noNetwork), .error:
            .dangerColor
        }
    }

    var disableButtons: Bool {
        if case .waitingForConnectivity(.noNetwork) = tunnelStatus.state {
            true
        } else {
            false
        }
    }

    var localizedTitleForSecureLabel: LocalizedStringKey {
        switch tunnelStatus.state {
        case .connecting, .reconnecting, .negotiatingEphemeralPeer:
            LocalizedStringKey("Connecting...")
        case .connected:
            LocalizedStringKey("Connected")
        case .disconnecting(.nothing):
            LocalizedStringKey("Disconnecting...")
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

    var accessibilityIdForSecureLabel: AccessibilityIdentifier {
        switch tunnelStatus.state {
        case .connected:
            .connectionStatusConnectedLabel
        case .connecting:
            .connectionStatusConnectingLabel
        default:
            .connectionStatusNotConnectedLabel
        }
    }

    var localizedAccessibilityLabelForSecureLabel: LocalizedStringKey {
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

    var localizedTitleForSelectLocationButton: LocalizedStringKey {
        switch tunnelStatus.state {
        case .disconnecting, .pendingReconnect, .disconnected, .waitingForConnectivity(.noNetwork):
            LocalizedStringKey(connectionName ?? "Select location")
        case .connecting, .connected, .reconnecting, .waitingForConnectivity(.noConnection),
             .negotiatingEphemeralPeer, .error:
            LocalizedStringKey("Switch location")
        }
    }

    var actionButton: TunnelActionButton {
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

    var inAddress: String? {
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
            let address = tunnelRelays.entry?.endpoint.ipv4Relay.ip
            ?? tunnelStatus.state.relays?.exit.endpoint.ipv4Relay.ip
        else {
            return nil
        }

        return "\(address)\(portAndTransport)"
    }

    var outAddressIpv4: String? {
        guard
            let outgoingConnectionInfo,
            let address = outgoingConnectionInfo.ipv4.exitIP ? outgoingConnectionInfo.ipv4.ip : nil
        else {
            return nil
        }

        return "\(address)"
    }

    var outAddressIpv6: String? {
        guard
            let outgoingConnectionInfo,
            let ipv6 = outgoingConnectionInfo.ipv6,
            let address = ipv6.exitIP ? ipv6.ip : nil
        else {
            return nil
        }

        return "\(address)"
    }
}
