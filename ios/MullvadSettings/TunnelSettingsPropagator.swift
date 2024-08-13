//
//  TunnelSettingsPropagator.swift
//  MullvadSettings
//
//  Created by Mojgan on 2024-08-09.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

public protocol SettingsPropagation {
    typealias SettingsHandler = (LatestTunnelSettings) -> Void
    var onNewSettings: SettingsHandler? { get set }
}

public protocol SettingsObserver: AnyObject {
    func settings(_ object: SettingsPropagation, didUpdateSettings settings: LatestTunnelSettings)
}

public class SettingsObserverBlock: SettingsObserver {
    public typealias DidUpdateSettingsHandler = (SettingsPropagation, LatestTunnelSettings) -> Void
    public var onNewSettings: DidUpdateSettingsHandler

    public init(didUpdateSettings: @escaping DidUpdateSettingsHandler) {
        self.onNewSettings = didUpdateSettings
    }

    public func settings(
        _ object: any SettingsPropagation,
        didUpdateSettings settings: LatestTunnelSettings
    ) {
        self.onNewSettings(object, settings)
    }
}

public final class TunnelSettingsListener: SettingsPropagation {
    public var onNewSettings: SettingsHandler?

    public init(onNewSettings: SettingsHandler? = nil) {
        self.onNewSettings = onNewSettings
    }
}

public class SettingsUpdater {
    /// Observers.
    private let observerList = ObserverList<SettingsObserver>()
    private var listener: SettingsPropagation

    public init(listener: SettingsPropagation) {
        self.listener = listener
        self.listener.onNewSettings = { [weak self] settings in
            guard let self else { return }
            self.observerList.notify {
                $0.settings(listener, didUpdateSettings: settings)
            }
        }
    }

    // MARK: - Multihop observations

    public func addObserver(_ observer: SettingsObserver) {
        observerList.append(observer)
    }

    public func removeObserver(_ observer: SettingsObserver) {
        observerList.remove(observer)
    }
}
