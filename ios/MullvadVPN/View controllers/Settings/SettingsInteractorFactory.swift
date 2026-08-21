// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadLogging
import MullvadREST
import MullvadSettings

final class SettingsInteractorFactory {
    private let apiProxy: APIQuerying
    private let relayCacheTracker: RelayCacheTracker
    private let ipOverrideRepository: IPOverrideRepositoryProtocol
    private let redactor: LogRedacting?

    let tunnelManager: TunnelManager

    init(
        tunnelManager: TunnelManager,
        apiProxy: APIQuerying,
        relayCacheTracker: RelayCacheTracker,
        ipOverrideRepository: IPOverrideRepositoryProtocol,
        redactor: LogRedacting? = nil
    ) {
        self.tunnelManager = tunnelManager
        self.apiProxy = apiProxy
        self.relayCacheTracker = relayCacheTracker
        self.ipOverrideRepository = ipOverrideRepository
        self.redactor = redactor
    }

    func makeVPNSettingsInteractor() -> VPNSettingsInteractor {
        VPNSettingsInteractor(tunnelManager: tunnelManager, relayCacheTracker: relayCacheTracker)
    }

    func makeProblemReportInteractor() -> ProblemReportInteractor {
        ProblemReportInteractor(apiProxy: apiProxy, tunnelManager: tunnelManager, redactor: redactor)
    }

    func makeSettingsInteractor() -> SettingsInteractor {
        SettingsInteractor(tunnelManager: tunnelManager)
    }

    func makeIPOverrideInteractor() -> IPOverrideInteractor {
        IPOverrideInteractor(repository: ipOverrideRepository, tunnelManager: tunnelManager)
    }
}
