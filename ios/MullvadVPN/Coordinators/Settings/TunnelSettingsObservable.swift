//
//  TunnelSettingsObservable.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-21.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings

protocol TunnelSettingsObservable<TunnelSetting>: ObservableObject {
    associatedtype TunnelSetting

    var value: TunnelSetting { get set }
    func evaluate(setting: TunnelSetting)
}

class MockTunnelSettingsViewModel<TunnelSetting>: TunnelSettingsObservable {
    @Published var value: TunnelSetting

    init(setting: TunnelSetting) {
        value = setting
    }

    func evaluate(setting: TunnelSetting) {}
}

protocol TunnelSettingsObserver<TunnelSetting>: TunnelSettingsObservable {
    associatedtype TunnelSetting

    var tunnelManager: TunnelManager { get }
    var tunnelObserver: TunnelObserver? { get set }

    init(tunnelManager: TunnelManager)
}
