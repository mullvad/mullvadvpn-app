//
//  DAITATunnelSettingsObservable.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-21.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

protocol DAITATunnelSettingsObservable: ObservableObject {
    var value: DAITASettings { get set }
}

class MockDAITATunnelSettingsViewModel: DAITATunnelSettingsObservable {
    @Published var value: DAITASettings

    init(daitaSettings: DAITASettings = DAITASettings()) {
        value = daitaSettings
    }
}

class DAITATunnelSettingsViewModel: DAITATunnelSettingsObserver, DAITATunnelSettingsObservable {}
