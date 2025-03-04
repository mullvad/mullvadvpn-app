//
//  PacketTunnelPathObserver.swift
//  PacketTunnel
//
//  Created by pronebird on 10/08/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadLogging
import MullvadTypes
import Network
import NetworkExtension
import PacketTunnelCore

final class PacketTunnelPathObserver: DefaultPathObserverProtocol, @unchecked Sendable {
    private let eventQueue: DispatchQueue
    private let pathMonitor: NWPathMonitor
    nonisolated(unsafe) let logger = Logger(label: "PacketTunnelPathObserver")

    private var gatewayConnection: NWConnection?
    private var started = false

    public var currentPathStatus: Network.NWPath.Status {
        pathMonitor.currentPath.status
    }

    init(eventQueue: DispatchQueue) {
        self.eventQueue = eventQueue

        pathMonitor = NWPathMonitor(prohibitedInterfaceTypes: [.other])
    }

    func start(_ body: @escaping @Sendable (Network.NWPath.Status) -> Void) {
        guard started == false else { return }
        defer { started = true }
        pathMonitor.pathUpdateHandler = { updatedPath in
            #if DEBUG
            var unsatisfiedReason = "<No value>"
            if updatedPath.status == .unsatisfied {
                unsatisfiedReason += updatedPath.unsatisfiedReasonDescription
            }
            var interfaceDebug = ""
            updatedPath.availableInterfaces.forEach { interfaceDebug += """
                        \($0.customDebugDescription)

            """ }
            let message = """
            Path available interfaces: \(interfaceDebug)
            Path status: \(updatedPath.status) Unsatisfied reason: \(unsatisfiedReason) Supports IPv4: \(
                updatedPath
                    .supportsIPv4
            )
            Supports IPv6: \(updatedPath.supportsIPv6) Supports DNS: \(updatedPath.supportsDNS) Is Constrained: \(
                updatedPath
                    .isConstrained
            )
            Is expensive: \(updatedPath.isExpensive) Gateways: \(updatedPath.gateways.map { $0.customDebugDescription })
            """
            self.logger.debug("\(message)")
            #endif
            body(updatedPath.status)
        }

        pathMonitor.start(queue: eventQueue)
    }

    func stop() {
        guard started == true else { return }
        defer { started = false }
        pathMonitor.pathUpdateHandler = nil
        pathMonitor.cancel()
    }
}
