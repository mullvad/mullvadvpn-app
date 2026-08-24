// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import WireGuardKit

struct WireGuardAdapterErrorWrapper: LocalizedError {
    let error: WireGuardAdapterError

    public var errorDescription: String? {
        switch error {
        case .cannotLocateTunnelFileDescriptor:
            return "Failure to locate tunnel file descriptor."

        case .invalidState:
            return "Failure to perform an operation in such state."

        case let .dnsResolution(resolutionErrors):
            let detailedErrorDescription =
                resolutionErrors
                .enumerated()
                .map { index, dnsResolutionError in
                    let errorDescription = dnsResolutionError.errorDescription ?? "???"

                    return "\(index): \(dnsResolutionError.address) \(errorDescription)"
                }
                .joined(separator: "\n")

            return "Failure to resolve endpoints:\n\(detailedErrorDescription)"

        case .setNetworkSettings:
            return "Failure to set network settings."

        case let .startWireGuardBackend(code):
            return "Failure to start WireGuard backend (error code: \(code))."
        case .noInterfaceIp:
            return "Interface has no IP address specified."
        case .noSuchTunnel:
            return "No such WireGuard tunnel"
        case .noTunnelVirtualInterface:
            return "Tunnel has no virtual (IAN) interface"
        case .icmpSocketNotOpen:
            return "ICMP socket not open"
        case let .internalError(code):
            return "Internal error \(code)"
        }
    }
}
