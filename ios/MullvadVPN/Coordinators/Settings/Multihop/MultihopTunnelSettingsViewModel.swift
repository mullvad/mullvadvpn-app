//
//  MultihopTunnelSettingsViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-21.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

protocol MultihopTunnelSettingsObservable: ObservableObject {
    var value: MultihopState { get set }
}

class MockMultihopTunnelSettingsViewModel: MultihopTunnelSettingsObservable {
    @Published var value: MultihopState

    init(multihopState: MultihopState = .off) {
        value = multihopState
    }
}

class MultihopTunnelSettingsViewModel: MultihopTunnelSettingsObserver, MultihopTunnelSettingsObservable {}
