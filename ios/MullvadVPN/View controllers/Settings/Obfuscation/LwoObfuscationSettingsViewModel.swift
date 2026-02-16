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
}

/** A simple mock view model for use in Previews and similar */
class MockLwoObfuscationSettingsViewModel: LwoObfuscationSettingsViewModel {
    @Published var value: WireGuardObfuscationLwoPort
    let portRanges: [[UInt16]] = []

    init(lwoPort: WireGuardObfuscationLwoPort = .automatic) {
        self.value = lwoPort
    }

    func commit() {}
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
}
