//
//  SettingsInteractorFactory.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST

final class SettingsInteractorFactory {
    private let storePaymentManager: StorePaymentManager
    private let tunnelManager: TunnelManager
    private let apiProxy: APIQuerying
    private let relayCacheTracker: RelayCacheTracker

    init(
        storePaymentManager: StorePaymentManager,
        tunnelManager: TunnelManager,
        apiProxy: APIQuerying,
        relayCacheTracker: RelayCacheTracker
    ) {
        self.storePaymentManager = storePaymentManager
        self.tunnelManager = tunnelManager
        self.apiProxy = apiProxy
        self.relayCacheTracker = relayCacheTracker
    }

    func makePreferencesInteractor() -> PreferencesInteractor {
        PreferencesInteractor(tunnelManager: tunnelManager, relayCacheTracker: relayCacheTracker)
    }

    func makeProblemReportInteractor() -> ProblemReportInteractor {
        ProblemReportInteractor(apiProxy: apiProxy, tunnelManager: tunnelManager)
    }

    func makeSettingsInteractor() -> SettingsInteractor {
        SettingsInteractor(tunnelManager: tunnelManager)
    }
}
