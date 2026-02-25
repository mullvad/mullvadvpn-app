//
//  LwoObfuscationSettingsViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-02-02.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

protocol LwoObfuscationSettingsViewModel: ObservableObject {
    var value: WireGuardObfuscationLwoPort { get set }
    var portRanges: [[UInt16]] { get }

    func commit()
    func validatePort(_ port: UInt16) -> WireGuardObfuscationLwoPort?
    func portRangesString() -> String
}

/** A simple mock view model for use in Previews and similar */
class MockLwoObfuscationSettingsViewModel: LwoObfuscationSettingsViewModel {
    @Published var value: WireGuardObfuscationLwoPort
    let portRanges: [[UInt16]] = []

    init(lwoPort: WireGuardObfuscationLwoPort = .automatic) {
        self.value = lwoPort
    }

    func commit() {}

    func validatePort(_ port: UInt16) -> WireGuardObfuscationLwoPort? {
        .custom(port)
    }

    func portRangesString() -> String {
        "Valid ranges: 1 - 1000, 5000 - 10000"
    }
}

/// ** The live view model which interfaces with the TunnelManager  */
class TunnelLwoObfuscationSettingsViewModel: TunnelObfuscationSettingsWatchingObservableObject<
    WireGuardObfuscationLwoPort
>,
LwoObfuscationSettingsViewModel
{
    let portRanges: [[UInt16]]

    init(tunnelManager: TunnelManager, portRanges: [[UInt16]]) {
        self.portRanges = portRanges

        super.init(
            tunnelManager: tunnelManager,
            keyPath: \.lwoPort
        )
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
        var string = "Valid ranges: "

        portRanges.enumerated().forEach { (index, range) in
            if let minPort = range.first, let maxPort = range.last {
                if index != 0 {
                    string.append(", ")
                }

                string.append(String(format: "%d - %d", minPort, maxPort))
            }
        }

        return NSLocalizedString(string, comment: "")
    }
}
