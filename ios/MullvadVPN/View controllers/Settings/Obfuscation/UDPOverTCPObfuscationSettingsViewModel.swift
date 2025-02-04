//
//  UDPOverTCPObfuscationSettingsViewModel.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2024-11-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

protocol UDPOverTCPObfuscationSettingsViewModel: ObservableObject {
    var value: WireGuardObfuscationUdpOverTcpPort { get set }

    func commit()
}

/** A simple mock view model for use in Previews and similar */
class MockUDPOverTCPObfuscationSettingsViewModel: UDPOverTCPObfuscationSettingsViewModel {
    @Published var value: WireGuardObfuscationUdpOverTcpPort

    init(udpTcpPort: WireGuardObfuscationUdpOverTcpPort = .automatic) {
        self.value = udpTcpPort
    }

    func commit() {}
}

/** The live view model which interfaces with the TunnelManager  */
class TunnelUDPOverTCPObfuscationSettingsViewModel: TunnelObfuscationSettingsWatchingObservableObject<
    WireGuardObfuscationUdpOverTcpPort
>,
    UDPOverTCPObfuscationSettingsViewModel {
    init(tunnelManager: TunnelManager) {
        super.init(
            tunnelManager: tunnelManager,
            keyPath: \.udpOverTcpPort
        )
    }
}
