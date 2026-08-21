// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadTypes
import NetworkExtension

class MockTunnel: TunnelProtocol, @unchecked Sendable {
    typealias TunnelManagerProtocol = SimulatorTunnelProviderManager

    var status: NEVPNStatus

    var isOnDemandEnabled: Bool

    var startDate: Date?

    var backgroundTaskProvider: BackgroundTaskProviding

    required init(tunnelProvider: TunnelManagerProtocol, backgroundTaskProvider: BackgroundTaskProviding) {
        status = .disconnected
        isOnDemandEnabled = false
        startDate = nil
        self.backgroundTaskProvider = backgroundTaskProvider
    }

    // Observers are currently unimplemented
    func addObserver(_ observer: TunnelStatusObserver) {}

    func removeObserver(_ observer: TunnelStatusObserver) {}

    func addBlockObserver(
        queue: DispatchQueue?,
        handler: @escaping (any TunnelProtocol, NEVPNStatus) -> Void
    ) -> TunnelStatusBlockObserver {
        fatalError("MockTunnel.addBlockObserver Not implemented")
    }

    func logFormat() -> String {
        ""
    }

    func saveToPreferences(_ completion: @escaping (Error?) -> Void) {
        completion(nil)
    }

    func removeFromPreferences(completion: @escaping (Error?) -> Void) {
        completion(nil)
    }

    func setConfiguration(_ configuration: TunnelConfiguration) {}

    func start(options: [String: NSObject]?) throws {
        startDate = Date()
    }

    func stop() {}

    func sendProviderMessage(_ messageData: Data, responseHandler: ((Data?) -> Void)?) throws {}
}
