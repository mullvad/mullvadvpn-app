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
    private let apiProxy: REST.APIProxy

    init(
        storePaymentManager: StorePaymentManager,
        tunnelManager: TunnelManager,
        apiProxy: REST.APIProxy
    ) {
        self.storePaymentManager = storePaymentManager
        self.tunnelManager = tunnelManager
        self.apiProxy = apiProxy
    }

    func makeAccountInteractor() -> AccountInteractor {
        return AccountInteractor(
            storePaymentManager: storePaymentManager,
            tunnelManager: tunnelManager
        )
    }

    func makePreferencesInteractor() -> PreferencesInteractor {
        return PreferencesInteractor(tunnelManager: tunnelManager)
    }

    func makeProblemReportInteractor() -> ProblemReportInteractor {
        return ProblemReportInteractor(apiProxy: apiProxy, tunnelManager: tunnelManager)
    }

    func makeSettingsInteractor() -> SettingsInteractor {
        return SettingsInteractor(tunnelManager: tunnelManager)
    }
}
