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
    var value: WireGuardObfuscationShadowsocksPort { get set }

    func commit()
}

/** A simple mock view model for use in Previews and similar */
class MockShadowsocksObfuscationSettingsViewModel: ShadowsocksObfuscationSettingsViewModel {
    @Published var value: WireGuardObfuscationShadowsocksPort

    init(shadowsocksPort: WireGuardObfuscationShadowsocksPort = .automatic) {
        self.value = shadowsocksPort
    }

    func commit() {}
}

/// ** The live view model which interfaces with the TunnelManager  */
class TunnelShadowsocksObfuscationSettingsViewModel: TunnelObfuscationSettingsWatchingObservableObject<
    WireGuardObfuscationShadowsocksPort
>,
    ShadowsocksObfuscationSettingsViewModel {
    init(tunnelManager: TunnelManager) {
        super.init(
            tunnelManager: tunnelManager,
            keyPath: \.shadowsocksPort
        )
    }
}
