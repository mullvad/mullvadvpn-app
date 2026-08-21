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
