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

    @Published var value: T

    init(tunnelManager: TunnelManager, keyPath: WritableKeyPath<WireGuardObfuscationSettings, T>) {
        self.tunnelManager = tunnelManager
        self.keyPath = keyPath
        self.value = tunnelManager.settings.wireGuardObfuscation[keyPath: keyPath]
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

    // Commit the temporarily stored value upstream
    func commit() {
        var obfuscationSettings = tunnelManager.settings.wireGuardObfuscation
        obfuscationSettings[keyPath: keyPath] = value
        tunnelManager.updateSettings([.obfuscation(obfuscationSettings)])
    }
}
