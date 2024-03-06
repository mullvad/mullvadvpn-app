//
//  TunnelStore+Stubs.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-03.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
@testable import MullvadVPN
import NetworkExtension

struct TunnelStoreStub: TunnelStoreProtocol {
    typealias TunnelType = TunnelStub
    func getPersistentTunnels() -> [TunnelType] {
        []
    }

    func createNewTunnel() -> TunnelType {
        TunnelStub(status: .invalid, isOnDemandEnabled: false)
    }
}

class DummyTunnelStatusObserver: TunnelStatusObserver {
    func tunnel(_ tunnel: any TunnelProtocol, didReceiveStatus status: NEVPNStatus) {}
}

final class TunnelStub: TunnelProtocol, Equatable {
    convenience init(tunnelProvider: TunnelProviderManagerType) {
        self.init(status: .invalid, isOnDemandEnabled: false)
    }

    static func == (lhs: TunnelStub, rhs: TunnelStub) -> Bool {
        ObjectIdentifier(lhs) == ObjectIdentifier(rhs)
    }

    init(status: NEVPNStatus, isOnDemandEnabled: Bool, startDate: Date? = nil) {
        self.status = status
        self.isOnDemandEnabled = isOnDemandEnabled
        self.startDate = startDate
    }

    func addObserver(_ observer: TunnelStatusObserver) {}

    func removeObserver(_ observer: TunnelStatusObserver) {}

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
