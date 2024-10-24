//
//  TunnelStore+Stubs.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-03.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import NetworkExtension

struct TunnelStoreStub: TunnelStoreProtocol {
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

final class TunnelStub: TunnelProtocol, Equatable {
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
