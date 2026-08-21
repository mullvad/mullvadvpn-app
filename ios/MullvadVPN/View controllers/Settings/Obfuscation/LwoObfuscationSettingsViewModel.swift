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
import SwiftUI

protocol LwoObfuscationSettingsViewModel {
    var port: Binding<WireGuardObfuscationLwoPort> { get }
    func validatePort(_ port: UInt16) -> WireGuardObfuscationLwoPort?
    func portRangesString() -> String
}

/** A simple mock view model for use in Previews and similar */
class MockLwoObfuscationSettingsViewModel: LwoObfuscationSettingsViewModel {
    var port: Binding<WireGuardObfuscationLwoPort>
    let portRanges: [[UInt16]] = []

    init(port: Binding<WireGuardObfuscationLwoPort>) {
        self.port = port
    }

    func validatePort(_ port: UInt16) -> WireGuardObfuscationLwoPort? {
        .custom(port)
    }

    func portRangesString() -> String {
        "Valid ranges: 1 - 1000, 5000 - 10000"
    }
}

/// ** The live view model which interfaces with the TunnelManager  */
class TunnelLwoObfuscationSettingsViewModel: LwoObfuscationSettingsViewModel {
    var port: Binding<WireGuardObfuscationLwoPort>
    let portRanges: [[UInt16]]

    init(port: Binding<WireGuardObfuscationLwoPort>, portRanges: [[UInt16]]) {
        self.port = port
        self.portRanges = portRanges
    }

    func validatePort(_ port: UInt16) -> WireGuardObfuscationLwoPort? {
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
