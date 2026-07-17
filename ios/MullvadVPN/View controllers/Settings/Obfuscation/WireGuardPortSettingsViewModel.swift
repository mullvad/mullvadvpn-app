//
//  WireGuardPortSettingsViewModel.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2026-07-28.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import MullvadTypes
import SwiftUI

protocol WireGuardPortSettingsViewModel {
    var port: Binding<WireGuardPort> { get }
    func validatePort(_ port: UInt16) -> WireGuardPort?
    func portRangesString() -> String
}

class WireGuardPortSettingsViewModelStub: WireGuardPortSettingsViewModel {
    var port: Binding<WireGuardPort>

    init(port: Binding<WireGuardPort>) {
        self.port = port
    }

    func validatePort(_ port: UInt16) -> WireGuardPort? {
        .custom(port)
    }

    func portRangesString() -> String {
        "Valid ranges: 1 - 1000, 5000 - 10000"
    }
}

class TunnelWireGuardPortSettingsViewModel: WireGuardPortSettingsViewModel {
    var port: Binding<WireGuardPort>
    let portRanges: [[UInt16]]

    init(port: Binding<WireGuardPort>, portRanges: [[UInt16]]) {
        self.port = port
        self.portRanges = portRanges
    }

    func validatePort(_ port: UInt16) -> WireGuardPort? {
        if port == 53 {
            return .port53
        }
        if port == 51820 {
            return .port51820
        }
        let portIsWithinValidRanges =
            portRanges
            .contains { range in
                if let minPort = range.first, let maxPort = range.last {
                    return (minPort...maxPort).contains(port)
                }
                return false
            }

        return portIsWithinValidRanges ? .custom(port) : nil
    }

    func portRangesString() -> String {
        var portsString = ""
        portRanges.enumerated().forEach { (index, range) in
            if let minPort = range.first, let maxPort = range.last {
                if index != 0 {
                    portsString.append(", ")
                }

                portsString.append(String(format: "%d - %d", minPort, maxPort))
            }
        }

        return String(format: NSLocalizedString("Valid ranges: %@", comment: ""), portsString)
    }
}
