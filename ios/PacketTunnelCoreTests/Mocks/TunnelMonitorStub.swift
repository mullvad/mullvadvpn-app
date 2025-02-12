//
//  TunnelMonitorStub.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 05/09/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network
import PacketTunnelCore

/// Tunnel monitor stub that can be configured with block handler to simulate a specific behavior.
class TunnelMonitorStub: TunnelMonitorProtocol, @unchecked Sendable {
    enum Command {
        case start, stop
    }

    class Dispatcher {
        typealias BlockHandler = (TunnelMonitorEvent, DispatchTimeInterval) -> Void

        private let block: BlockHandler
        init(_ block: @escaping BlockHandler) {
            self.block = block
        }

        func send(_ event: TunnelMonitorEvent, after delay: DispatchTimeInterval = .never) {
            block(event, delay)
        }
    }

    typealias EventHandler = (TunnelMonitorEvent) -> Void
    typealias SimulationHandler = (Command, Dispatcher) -> Void

    private var eventHandler: AsyncStream<TunnelMonitorEvent>.Continuation?
    let eventStream: AsyncStream<TunnelMonitorEvent>
    private let simulationBlock: SimulationHandler

    init(_ simulationBlock: @escaping SimulationHandler) {
        self.simulationBlock = simulationBlock

        var innerContinuation: AsyncStream<TunnelMonitorEvent>.Continuation?
        let stream = AsyncStream<TunnelMonitorEvent> { continuation in
            innerContinuation = continuation
        }
        self.eventStream = stream
        self.eventHandler = innerContinuation
    }

    func start(probeAddress: IPv4Address) async {
        sendCommand(.start)
    }

    func stop() async {
        sendCommand(.stop)
    }

    func wake() async {}

    func sleep() async {}

    func handleNetworkPathUpdate(_ networkPath: Network.NWPath.Status) async {}

    func dispatch(_ event: TunnelMonitorEvent, after delay: DispatchTimeInterval = .never) {
        if case .never = delay {
            eventHandler?.yield(event)
        } else {
            DispatchQueue.main.asyncAfter(wallDeadline: .now() + delay) { [weak self] in
                self?.eventHandler?.yield(event)
            }
        }
    }

    private func sendCommand(_ command: Command) {
        let dispatcher = Dispatcher { [weak self] event, delay in
            self?.dispatch(event, after: delay)
        }
        simulationBlock(command, dispatcher)
    }
}

extension TunnelMonitorStub {
    /// Returns a mock of tunnel monitor that always reports that connection is established after a short delay.
    static func nonFallible() -> TunnelMonitorStub {
        TunnelMonitorStub { command, dispatcher in
            if case .start = command {
                dispatcher.send(.connectionEstablished, after: .milliseconds(10))
            }
        }
    }
}
