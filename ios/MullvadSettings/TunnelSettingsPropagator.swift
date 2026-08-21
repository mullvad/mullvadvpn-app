// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
