//
//  WireGuardAdapterError+Localization.swift
//  PacketTunnel
//
//  Created by pronebird on 14/07/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import WireGuardKit

extension WireGuardAdapterError: LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .cannotLocateTunnelFileDescriptor:
            return "Failure to locate tunnel file descriptor."

        case .invalidState:
            return "Failure to perform an operation in such state."

        case let .dnsResolution(resolutionErrors):
            let detailedErrorDescription = resolutionErrors
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
        }
    }
}
