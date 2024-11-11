//
//  TunnelObfuscationSettingsWatchingObservableObject.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2024-11-07.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

/// a generic ObservableObject that binds to obfuscation settings in TunnelManager.
/// Used as the basis for ViewModels for SwiftUI interfaces for these settings.

class TunnelObfuscationSettingsWatchingObservableObject<T: Equatable>: ObservableObject {
    let tunnelManager: TunnelManager
    let keyPath: WritableKeyPath<WireGuardObfuscationSettings, T>
    private var tunnelObserver: TunnelObserver?

    // this is essentially @Published from scratch
    var value: T {
        willSet(newValue) {
            guard newValue != self.value else { return }
            objectWillChange.send()
            var obfuscationSettings = tunnelManager.settings.wireGuardObfuscation
            obfuscationSettings[keyPath: keyPath] = newValue
            tunnelManager.updateSettings([.obfuscation(obfuscationSettings)])
        }
    }

    init(tunnelManager: TunnelManager, keyPath: WritableKeyPath<WireGuardObfuscationSettings, T>, _ initialValue: T) {
        self.tunnelManager = tunnelManager
        self.keyPath = keyPath
        self.value = initialValue
        tunnelObserver =
            TunnelBlockObserver(didUpdateTunnelSettings: { [weak self] _, newSettings in
                guard let self else { return }
                updateValueFromSettings(newSettings.wireGuardObfuscation)
            })
    }

    private func updateValueFromSettings(_ settings: WireGuardObfuscationSettings) {
        let newValue = settings[keyPath: keyPath]
        if value != newValue {
            value = newValue
        }
    }
}
