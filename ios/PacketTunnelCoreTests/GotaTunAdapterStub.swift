//
//  GotaTunAdapterStub.swift
//  PacketTunnelCoreTests
//
//  Created by Emīls on 2026-08-13.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import PacketTunnelCore
import os

/// Configurable stub for `GotaTunAdapterProtocol`.
/// Schedules callbacks based on the configured outcome.
final class GotaTunAdapterStub: GotaTunAdapterProtocol, Sendable {
    enum Outcome: Sendable {
        case connected(after: Duration = .zero)
        case timeout(after: Duration = .zero)
        case error(GotaTunError, after: Duration = .zero)
        case connectedThenTimeout(connectedAfter: Duration = .zero, timeoutAfter: Duration = .zero)
        /// Never reports anything; the tunnel stays in whatever state it was started from.
        case never

        /// The callbacks this outcome reports, in order, each after its own delay.
        var steps: [(delay: Duration, report: @Sendable (GotaTunCallbackHandler) -> Void)] {
            switch self {
            case .never:
                return []
            case let .connected(after):
                return [(after, { $0.onConnected() })]
            case let .timeout(after):
                return [(after, { $0.onTimeout() })]
            case let .error(error, after):
                return [(after, { $0.onError(error) })]
            case let .connectedThenTimeout(connectedAfter, timeoutAfter):
                return [(connectedAfter, { $0.onConnected() }), (timeoutAfter, { $0.onTimeout() })]
            }
        }
    }

    private struct State {
        var lastConfig: GotaTunConfig?
        var callbackHandler: GotaTunCallbackHandler?
        var callbackTask: Task<Void, Never>?
        var isStopped = false
        var recycleUdpSocketsCount = 0
        var onRecycleUdpSockets: (@Sendable () -> Void)?
    }

    let outcome: Outcome
    private let state = OSAllocatedUnfairLock(initialState: State())

    var lastConfig: GotaTunConfig? { state.withLock { $0.lastConfig } }
    /// Retained after `stopTunnel` so tests can simulate a callback that was already in
    /// flight when the adapter was stopped or replaced.
    var callbackHandler: GotaTunCallbackHandler? { state.withLock { $0.callbackHandler } }
    var isStopped: Bool { state.withLock { $0.isStopped } }
    var recycleUdpSocketsCount: Int { state.withLock { $0.recycleUdpSocketsCount } }

    var onRecycleUdpSockets: (@Sendable () -> Void)? {
        get { state.withLock { $0.onRecycleUdpSockets } }
        set { state.withLock { $0.onRecycleUdpSockets = newValue } }
    }

    init(outcome: Outcome = .connected()) {
        self.outcome = outcome
    }

    func startTunnel(config: GotaTunConfig, callbackHandler: GotaTunCallbackHandler) throws {
        state.withLock {
            $0.lastConfig = config
            $0.callbackHandler = callbackHandler
            $0.isStopped = false
            $0.callbackTask = Task { [outcome, state] in
                for step in outcome.steps {
                    if step.delay > .zero {
                        try? await Task.sleep(for: step.delay)
                    }
                    guard !state.withLock({ $0.isStopped }) else { return }
                    step.report(callbackHandler)
                }
            }
        }
    }

    func stopTunnel() {
        state.withLock {
            $0.isStopped = true
            $0.callbackTask?.cancel()
            $0.callbackTask = nil
        }
    }

    func recycleUdpSockets() {
        let onRecycle = state.withLock {
            $0.recycleUdpSocketsCount += 1
            return $0.onRecycleUdpSockets
        }
        onRecycle?()
    }

    func suspendTunnel() {}

    func wakeTunnel() {}
}

/// Stub for `TunnelProviderDelegate` that records what the actor asked of the provider.
actor TunnelProviderDelegateStub: TunnelProviderDelegate {
    let tunnelFileDescriptor: Int32?
    /// Error thrown by `applyNetworkSettings`, if any.
    var applyError: Error?

    var appliedSettings: [TunnelInterfaceSettings] = []
    var reassertCount = 0

    init(tunnelFileDescriptor: Int32? = 0) {
        self.tunnelFileDescriptor = tunnelFileDescriptor
    }

    func applyNetworkSettings(_ settings: TunnelInterfaceSettings) async throws {
        if let applyError { throw applyError }
        appliedSettings.append(settings)
    }

    func reassertTunnel() async {
        reassertCount += 1
    }
}

/// Factory that returns a sequence of adapters with configurable outcomes.
final class GotaTunAdapterFactoryStub: GotaTunAdapterFactory {
    private let outcomes: [GotaTunAdapterStub.Outcome]
    private let created = OSAllocatedUnfairLock(initialState: [GotaTunAdapterStub]())

    var adaptersCreated: [GotaTunAdapterStub] { created.withLock { $0 } }

    /// Create a factory that returns adapters with the given outcomes in order.
    /// Once outcomes are exhausted, the last outcome is reused.
    init(outcomes: [GotaTunAdapterStub.Outcome]) {
        precondition(!outcomes.isEmpty)
        self.outcomes = outcomes
    }

    /// Convenience: all adapters use the same outcome.
    convenience init(outcome: GotaTunAdapterStub.Outcome = .connected()) {
        self.init(outcomes: [outcome])
    }

    func makeAdapter() -> GotaTunAdapterProtocol {
        created.withLock {
            let adapter = GotaTunAdapterStub(outcome: outcomes[min($0.count, outcomes.count - 1)])
            $0.append(adapter)
            return adapter
        }
    }
}
