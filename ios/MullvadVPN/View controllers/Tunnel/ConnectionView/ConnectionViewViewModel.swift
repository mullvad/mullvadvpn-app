//
//  ConnectionViewViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-09.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadREST
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

    @Published var relayConstraints: RelayConstraints
    let destinationDescriber: DestinationDescribing

    var tunnelIsConnected: Bool {
        if case .connected = tunnelStatus.state {
            true
        } else {
            false
        }
    }

    var connectionName: String? {
        if case let .only(loc) = relayConstraints.exitLocations {
            return destinationDescriber.describe(loc)
        }
        return nil
    }

    init(
        tunnelStatus: TunnelStatus,
        relayConstraints: RelayConstraints,
        relayCache: RelayCacheProtocol,
        customListRepository: CustomListRepositoryProtocol
    ) {
        self.tunnelStatus = tunnelStatus
        self.relayConstraints = relayConstraints
        self.destinationDescriber = DestinationDescriber(
            relayCache: relayCache,
            customListRepository: customListRepository
        )
    }

    func update(tunnelStatus: TunnelStatus) {
        self.tunnelStatus = tunnelStatus
    }
}

extension ConnectionViewViewModel {
    var showsConnectionDetails: Bool {
        switch tunnelStatus.state {
        case .connecting, .reconnecting, .negotiatingEphemeralPeer,
             .connected, .pendingReconnect:
            true
        case .disconnecting, .disconnected, .waitingForConnectivity, .error:
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

    var titleForSecureLabel: String {
        switch tunnelStatus.state {
        case .connecting, .reconnecting, .negotiatingEphemeralPeer:
            NSLocalizedString("TUNNEL_STATE_CONNECTING", value: "Connecting...", comment: "")
        case .connected:
            NSLocalizedString("TUNNEL_STATE_CONNECTED", value: "Connected", comment: "")
        case .disconnecting(.nothing):
            NSLocalizedString("TUNNEL_STATE_DISCONNECTING", value: "Disconnecting...", comment: "")
        case .disconnecting(.reconnect), .pendingReconnect:
            NSLocalizedString("TUNNEL_STATE_RECONNECTING", value: "Reconnecting", comment: "")
        case .disconnected:
            NSLocalizedString("TUNNEL_STATE_DISCONNECTED", value: "Disconnected", comment: "")
        case .waitingForConnectivity(.noConnection), .error:
            NSLocalizedString("TUNNEL_STATE_BLOCKED", value: "Blocked connection", comment: "")
        case .waitingForConnectivity(.noNetwork):
            NSLocalizedString("TUNNEL_STATE_NO_NETWORK", value: "No network", comment: "")
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

    var accessibilityLabelForSecureLabel: String {
        switch tunnelStatus.state {
        case .disconnected, .waitingForConnectivity, .disconnecting, .pendingReconnect, .error:
            return titleForSecureLabel

        case let .connected(tunnelInfo, _, _):
            return String(
                format: NSLocalizedString(
                    "SECURE_LABEL_CONNECTED_TO_FORMAT",
                    value: "Connected to %@, %@",
                    comment: ""
                ),
                tunnelInfo.exit.location.city,
                tunnelInfo.exit.location.country
            )

        case let .connecting(tunnelInfo, _, _):
            if let tunnelInfo {
                return String(
                    format: NSLocalizedString(
                        "SECURE_LABEL_CONNECTING_TO_FORMAT",
                        value: "Connected to %@, %@",
                        comment: ""
                    ),
                    tunnelInfo.exit.location.city,
                    tunnelInfo.exit.location.country
                )
            } else {
                return titleForSecureLabel
            }

        case let .reconnecting(tunnelInfo, _, _),
             let .negotiatingEphemeralPeer(tunnelInfo, _, _, _):
            return String(
                format: NSLocalizedString(
                    "SECURE_LABEL_RECONNECTING_TO_FORMAT",
                    value: "Connected to %@, %@",
                    comment: ""
                ),
                tunnelInfo.exit.location.city,
                tunnelInfo.exit.location.country
            )
        }
    }

    var titleForSelectLocationButton: String {
        switch tunnelStatus.state {
        case .disconnecting, .pendingReconnect, .disconnected, .waitingForConnectivity(.noNetwork):
            connectionName ?? NSLocalizedString("SWITCH_LOCATION_BUTTON_TITLE", comment: "")
        case .connecting, .connected, .reconnecting, .waitingForConnectivity(.noConnection),
             .negotiatingEphemeralPeer, .error:
            NSLocalizedString("SWITCH_LOCATION_BUTTON_TITLE", comment: "")
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

    var titleForCountryAndCity: String? {
        guard let tunnelRelays = tunnelStatus.state.relays else {
            return nil
        }
        return String(
            format: NSLocalizedString(
                "TUNNEL_LOCATION_FORMAT",
                value: "%@, %@",
                comment: ""
            ),
            tunnelRelays.exit.location.country,
            tunnelRelays.exit.location.city
        )
    }

    var titleForServer: String? {
        guard let tunnelRelays = tunnelStatus.state.relays else {
            return nil
        }

        let exitName = tunnelRelays.exit.hostname
        let entryName = tunnelRelays.entry?.hostname

        if let entry = entryName {
            return String(
                format: NSLocalizedString(
                    "EXIT_VIA_ENTRY_FORMAT",
                    value: "%@ via %@",
                    comment: ""
                ),
                exitName,
                entry
            )
        } else {
            return String(
                format: NSLocalizedString(
                    "EXIT_ONLY_FORMAT", value: "%@", comment: ""
                ),
                exitName
            )
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
