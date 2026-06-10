//
//  MockTunnel.swift
//  MullvadVPNTests
//
//  Created by Andrew Bulhak on 2024-02-05.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import NetworkExtension

class MockTunnel: TunnelProtocol, @unchecked Sendable {
    typealias TunnelManagerProtocol = SimulatorTunnelProviderManager

    var status: NEVPNStatus

    var isOnDemandEnabled: Bool

    var startDate: Date?

    var backgroundTaskProvider: BackgroundTaskProviding

    /// Test hook invoked by `sendProviderMessage`. Defaults to not responding at all.
    var onSendProviderMessage: ((Data, ((Data?) -> Void)?) throws -> Void)?

    private let observerList = ObserverList<any TunnelStatusObserver>()

    required init(tunnelProvider: TunnelManagerProtocol, backgroundTaskProvider: BackgroundTaskProviding) {
        status = .disconnected
        isOnDemandEnabled = false
        startDate = nil
        self.backgroundTaskProvider = backgroundTaskProvider
    }

    /// Set the status and notify observers, as a real tunnel would on NEVPNStatusDidChange.
    func simulateStatusChange(_ newStatus: NEVPNStatus) {
        status = newStatus
        observerList.notify { observer in
            observer.tunnel(self, didReceiveStatus: newStatus)
        }
    }

    func addObserver(_ observer: TunnelStatusObserver) {
        observerList.append(observer)
    }

    func removeObserver(_ observer: TunnelStatusObserver) {
        observerList.remove(observer)
    }

    func addBlockObserver(
        queue: DispatchQueue?,
        handler: @escaping (any TunnelProtocol, NEVPNStatus) -> Void
    ) -> TunnelStatusBlockObserver {
        let observer = TunnelStatusBlockObserver(tunnel: self, queue: queue, handler: handler)

        addObserver(observer)

        return observer
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

    func sendProviderMessage(_ messageData: Data, responseHandler: ((Data?) -> Void)?) throws {
        try onSendProviderMessage?(messageData, responseHandler)
    }
}
