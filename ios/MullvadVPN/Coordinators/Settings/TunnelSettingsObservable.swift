// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
