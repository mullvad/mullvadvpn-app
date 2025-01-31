//
//  PacketTunnelPathObserver.swift
//  PacketTunnel
//
//  Created by pronebird on 10/08/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadTypes
import Network
import NetworkExtension
import PacketTunnelCore

final class PacketTunnelPathObserver: DefaultPathObserverProtocol, @unchecked Sendable {
    private weak var packetTunnelProvider: NEPacketTunnelProvider?
    private let eventQueue: DispatchQueue
    private let pathMonitor: NWPathMonitor

    private var gatewayConnection: NWConnection?

    public var currentPathStatus: Network.NWPath.Status {
        pathMonitor.currentPath.status
    }

    init(packetTunnelProvider: NEPacketTunnelProvider, eventQueue: DispatchQueue) {
        self.packetTunnelProvider = packetTunnelProvider
        self.eventQueue = eventQueue

        pathMonitor = NWPathMonitor(prohibitedInterfaceTypes: [.other])
    }

    func start(_ body: @escaping @Sendable (Network.NWPath.Status) -> Void) {
        pathMonitor.pathUpdateHandler = { updatedPath in
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
            print(message)
            body(updatedPath.status)
        }

        pathMonitor.start(queue: eventQueue)
    }

    func stop() {
//        pathMonitor.pathUpdateHandler = nil
//        pathMonitor.cancel()
    }
}
