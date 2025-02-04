//
//  TunnelSettingsPropagator.swift
//  MullvadSettings
//
//  Created by Mojgan on 2024-08-09.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

public protocol SettingsPropagation: Sendable {
    typealias SettingsHandler = (LatestTunnelSettings) -> Void
    var onNewSettings: SettingsHandler? { get set }
}

public protocol SettingsObserver: AnyObject {
    func didUpdateSettings(_ settings: LatestTunnelSettings)
}

public class SettingsObserverBlock: SettingsObserver {
    public typealias DidUpdateSettingsHandler = (LatestTunnelSettings) -> Void
    public var onNewSettings: DidUpdateSettingsHandler

    public init(didUpdateSettings: @escaping DidUpdateSettingsHandler) {
        self.onNewSettings = didUpdateSettings
    }

    public func didUpdateSettings(_ settings: LatestTunnelSettings) {
        self.onNewSettings(settings)
    }
}

public final class TunnelSettingsListener: SettingsPropagation, @unchecked Sendable {
    public var onNewSettings: SettingsHandler?

    public init(onNewSettings: SettingsHandler? = nil) {
        self.onNewSettings = onNewSettings
    }
}

public final class SettingsUpdater: Sendable {
    /// Observers.
    private let observerList = ObserverList<SettingsObserver>()
    nonisolated(unsafe) private var listener: SettingsPropagation

    public init(listener: SettingsPropagation) {
        self.listener = listener
        self.listener.onNewSettings = { [weak self] settings in
            guard let self else { return }
            self.observerList.notify {
                $0.didUpdateSettings(settings)
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
