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
                NSLocalizedString("Creating quantum secure connection", comment: "")
            } else {
                NSLocalizedString("Creating secure connection", comment: "")
            }

        case let .negotiatingEphemeralPeer(_, _, isPostQuantum, _):
            if isPostQuantum {
                NSLocalizedString("Creating quantum secure connection", comment: "")
            } else {
                NSLocalizedString("Creating secure connection", comment: "")
            }

        case let .connected(_, isPostQuantum, _):
            if isPostQuantum {
                NSLocalizedString("Quantum secure connection", comment: "")
            } else {
                NSLocalizedString("Connected", comment: "")
            }

        case .disconnecting(.nothing):
            NSLocalizedString("Disconnecting", comment: "")

        case .disconnecting(.reconnect), .pendingReconnect:
            NSLocalizedString("Reconnecting", comment: "")

        case .disconnected:
            NSLocalizedString("Unsecured connection", comment: "")

        case .waitingForConnectivity(.noConnection), .error:
            NSLocalizedString("Blocked connection", comment: "")

        case .waitingForConnectivity(.noNetwork):
            NSLocalizedString("No network", comment: "")
        }
    }

    var localizedTitleForSelectLocationButton: String {
        switch self {
        case .disconnecting(.reconnect), .pendingReconnect:
            NSLocalizedString("Select location", comment: "")

        case .disconnected, .disconnecting(.nothing):
            NSLocalizedString("Select location", comment: "")

        case .connecting, .connected, .reconnecting, .waitingForConnectivity, .error:
            NSLocalizedString("Switch location", comment: "")

        case .negotiatingEphemeralPeer:
            NSLocalizedString("Switch location", comment: "")
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
                    format: NSLocalizedString("Quantum secure connection. Connected to %@, %@", comment: ""),
                    tunnelInfo.exit.location.city,
                    tunnelInfo.exit.location.country
                )
            } else {
                String(
                    format: NSLocalizedString("Secure connection. Connected to %@, %@", comment: ""),
                    tunnelInfo.exit.location.city,
                    tunnelInfo.exit.location.country
                )
            }

        case .disconnected:
            NSLocalizedString("Unsecured connection", comment: "")
        case let .reconnecting(tunnelInfo, _, _):
            String(
                format: NSLocalizedString("Reconnecting to %@, %@", comment: ""),
                tunnelInfo.exit.location.city,
                tunnelInfo.exit.location.country
            )

        case .waitingForConnectivity(.noConnection), .error:
            NSLocalizedString("Blocked connection", comment: "")

        case .waitingForConnectivity(.noNetwork):
            NSLocalizedString("No network", comment: "")

        case .disconnecting(.nothing):
            NSLocalizedString("Disconnecting", comment: "")

        case .disconnecting(.reconnect), .pendingReconnect:
            NSLocalizedString("Reconnecting", comment: "")
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
                "%@ via %@\(usingDaita ? " using DAITA" : "")",
                comment: ""
            ), exitName, entryName)
        } else {
            String(format: NSLocalizedString(
                "%@\(usingDaita ? " using DAITA" : "")",
                comment: ""
            ), exitName)
        }
    }

    func secureConnectionLabel(isPostQuantum: Bool) -> String {
        if isPostQuantum {
            NSLocalizedString("Creating quantum secure connection", comment: "")
        } else {
            NSLocalizedString("Creating secure connection", comment: "")
        }
    }
}
