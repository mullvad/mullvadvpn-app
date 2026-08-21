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

struct TunnelStoreStub: TunnelStoreProtocol, Sendable {
    typealias TunnelType = TunnelStub
    let backgroundTaskProvider: any BackgroundTaskProviding
    func getPersistentTunnels() -> [TunnelType] {
        []
    }

    func createNewTunnel() -> TunnelType {
        TunnelStub(backgroundTaskProvider: backgroundTaskProvider, status: .invalid, isOnDemandEnabled: false)
    }
}

class DummyTunnelStatusObserver: TunnelStatusObserver {
    func tunnel(_ tunnel: any TunnelProtocol, didReceiveStatus status: NEVPNStatus) {}
}

final class TunnelStub: TunnelProtocol, Equatable, @unchecked Sendable {
    typealias TunnelManagerProtocol = SimulatorTunnelProviderManager

    static func == (lhs: TunnelStub, rhs: TunnelStub) -> Bool {
        ObjectIdentifier(lhs) == ObjectIdentifier(rhs)
    }

    convenience init(
        tunnelProvider: SimulatorTunnelProviderManager,
        backgroundTaskProvider: any BackgroundTaskProviding
    ) {
        self.init(backgroundTaskProvider: backgroundTaskProvider, status: .invalid, isOnDemandEnabled: false)
    }

    init(
        backgroundTaskProvider: any BackgroundTaskProviding,
        status: NEVPNStatus,
        isOnDemandEnabled: Bool,
        startDate: Date? = nil
    ) {
        self.status = status
        self.isOnDemandEnabled = isOnDemandEnabled
        self.startDate = startDate
        self.backgroundTaskProvider = backgroundTaskProvider
    }

    func addObserver(_ observer: TunnelStatusObserver) {}

    func removeObserver(_ observer: TunnelStatusObserver) {}

    var backgroundTaskProvider: any BackgroundTaskProviding

    var status: NEVPNStatus

    var isOnDemandEnabled: Bool

    var startDate: Date?

    func addBlockObserver(
        queue: DispatchQueue?,
        handler: @escaping (any TunnelProtocol, NEVPNStatus) -> Void
    ) -> TunnelStatusBlockObserver {
        TunnelStatusBlockObserver(tunnel: self, queue: queue, handler: handler)
    }

    func logFormat() -> String {
        ""
    }

    func saveToPreferences(_ completion: @escaping (Error?) -> Void) {}

    func removeFromPreferences(completion: @escaping (Error?) -> Void) {}

    func setConfiguration(_ configuration: TunnelConfiguration) {}

    func start(options: [String: NSObject]?) throws {}

    func stop() {}

    func sendProviderMessage(_ messageData: Data, responseHandler: ((Data?) -> Void)?) throws {}
}
