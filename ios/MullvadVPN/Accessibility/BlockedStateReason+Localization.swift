//
//  BlockedStateReason+Localization.swift
//  MullvadVPN
//
//  Created on 2026-02-16.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import PacketTunnelCore

extension BlockedStateReason {
    var localizedReason: String {
        switch self {
        case .outdatedSchema:
            NSLocalizedString(
                "Unable to start tunnel connection after update. Please disconnect and reconnect.",
                comment: ""
            )
        case .noRelaysSatisfyingFilterConstraints:
            NSLocalizedString("No servers match your location filter. Try changing filter settings.", comment: "")
        case .multihopEntryEqualsExit:
            NSLocalizedString(
                "The entry and exit servers cannot be the same. Try changing one to a new server or location.",
                comment: ""
            )
        case .noRelaysSatisfyingDaitaConstraints:
            NSLocalizedString(
                "No DAITA compatible servers match your location settings. Try changing location.",
                comment: ""
            )
        case .noRelaysSatisfyingObfuscationSettings:
            NSLocalizedString(
                "No servers match your obfuscation settings. Try changing location or obfuscation method.",
                comment: ""
            )
        case .noRelaysSatisfyingConstraints:
            NSLocalizedString("No servers match your settings, try changing server or other settings.", comment: "")
        case .noRelaysSatisfyingPortConstraints:
            NSLocalizedString(
                "The selected WireGuard port is not supported, please change it under **VPN settings**.",
                comment: ""
            )
        case .noRelaysSatisfyingObfuscationPortConstraints:
            NSLocalizedString(
                "The selected obfuscation port is not supported, please change it under **VPN settings**.",
                comment: ""
            )
        case .invalidAccount:
            NSLocalizedString(
                "You are logged in with an invalid account number. Please log out and try another one.",
                comment: ""
            )
        case .deviceLoggedOut:
            NSLocalizedString("Unable to authenticate account. Please log out and log back in.", comment: "")
        default:
            NSLocalizedString("Unable to start tunnel connection. Please send a problem report.", comment: "")
        }
    }
}
