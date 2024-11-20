//
//  ShadowsocksObfuscationSettingsViewModel.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2024-11-07.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

protocol ShadowsocksObfuscationSettingsViewModel: ObservableObject {
    var value: WireGuardObfuscationShadowsockPort { get set }
}

/** A simple mock view model for use in Previews and similar */
class MockShadowsocksObfuscationSettingsViewModel: ShadowsocksObfuscationSettingsViewModel {
    @Published var value: WireGuardObfuscationShadowsockPort

    init(shadowsocksPort: WireGuardObfuscationShadowsockPort = .automatic) {
        self.value = shadowsocksPort
    }
}

/// ** The live view model which interfaces with the TunnelManager  */
class TunnelShadowsocksObfuscationSettingsViewModel: TunnelObfuscationSettingsWatchingObservableObject<
    WireGuardObfuscationShadowsockPort
>,
    ShadowsocksObfuscationSettingsViewModel {
    init(tunnelManager: TunnelManager) {
        super.init(
            tunnelManager: tunnelManager,
            keyPath: \.shadowsocksPort,
            tunnelManager.settings.wireGuardObfuscation.shadowsocksPort
        )
    }
}
