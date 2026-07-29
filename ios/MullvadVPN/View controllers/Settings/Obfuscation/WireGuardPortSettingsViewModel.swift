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

protocol WireGuardPortSettingsViewModel: ObservableObject {
    var value: RelayConstraint<UInt16> { get set }
    var selectedOption: WireGuardPort { get set }
    var portRanges: [[UInt16]] { get }

    func commit()
    func validatePort(_ port: UInt16) -> WireGuardPort?
    func portRangesString() -> String
}

/** A simple mock view model for use in Previews and similar */
class MockWireGuardPortSettingsViewModel: WireGuardPortSettingsViewModel {
    @Published var value: RelayConstraint<UInt16>
    @Published var selectedOption: WireGuardPort

    let portRanges: [[UInt16]] = []

    init(customPort: RelayConstraint<UInt16> = .any, option: WireGuardPort) {
        self.value = customPort
        self.selectedOption = option
    }

    func commit() {}

    func validatePort(_ port: UInt16) -> WireGuardPort? {
        .custom(port)
    }

    func portRangesString() -> String {
        "Valid ranges: 1 - 1000, 5000 - 10000"
    }
}

/// ** The live view model which interfaces with the TunnelManager  */
class TunnelWireGuardPortSettingsViewModel: TunnelRelayConstraintsWatchingObservableObject<RelayConstraint<UInt16>>,
    WireGuardPortSettingsViewModel
{
    let portRanges: [[UInt16]]
    @Published var selectedOption: WireGuardPort

    init(tunnelManager: TunnelManager, option: WireGuardPort, portRanges: [[UInt16]]) {
        self.portRanges = portRanges
        self.selectedOption = option

        super.init(
            tunnelManager: tunnelManager,
            keyPath: \.port
        )
    }

    override func commit() {
        value =
            switch selectedOption {
            case .automatic: .any
            case .port51820: .only(51820)
            case .port53: .only(53)
            case let .custom(port): .only(port)
            }
        super.commit()
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
