//
//  GotaTunAdapterStub.swift
//  PacketTunnelCoreTests
//
//  Created by Emīls on 2026-08-13.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import PacketTunnelCore

/// Configurable stub for `GotaTunAdapterProtocol`.
/// Schedules callbacks based on the configured outcome.
final class GotaTunAdapterStub: GotaTunAdapterProtocol, @unchecked Sendable {
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

    let outcome: Outcome
    private(set) var lastConfig: GotaTunConfig?
    /// Retained after `stopTunnel` so tests can simulate a callback that was already in
    /// flight when the adapter was stopped or replaced.
    private(set) var callbackHandler: GotaTunCallbackHandler?
    private var callbackTask: Task<Void, Never>?
    private(set) var isStopped = false

    init(outcome: Outcome = .connected()) {
        self.outcome = outcome
    }

    func startTunnel(config: GotaTunConfig, callbackHandler: GotaTunCallbackHandler) throws {
        self.lastConfig = config
        self.callbackHandler = callbackHandler
        isStopped = false

        callbackTask = Task { [weak self, outcome] in
            for step in outcome.steps {
                if step.delay > .zero {
                    try? await Task.sleep(for: step.delay)
                }
                guard !Task.isCancelled, let self, !self.isStopped else { return }
                step.report(callbackHandler)
            }
        }
    }

    func stopTunnel() {
        isStopped = true
        callbackTask?.cancel()
        callbackTask = nil
    }

    private let counterLock = NSLock()
    private var _recycleUdpSocketsCount = 0
    private var _suspendCount = 0
    private var _wakeCount = 0

    var onRecycleUdpSockets: (@Sendable () -> Void)?

    var recycleUdpSocketsCount: Int { counterLock.withLock { _recycleUdpSocketsCount } }
    var suspendCount: Int { counterLock.withLock { _suspendCount } }
    var wakeCount: Int { counterLock.withLock { _wakeCount } }

    func recycleUdpSockets() {
        counterLock.withLock { _recycleUdpSocketsCount += 1 }
        onRecycleUdpSockets?()
    }

    func suspendTunnel() {
        counterLock.withLock { _suspendCount += 1 }
    }

    func wakeTunnel() {
        counterLock.withLock { _wakeCount += 1 }
    }
}

/// Stub for `TunnelProviderDelegate` that records what the actor asked of the provider.
final class TunnelProviderDelegateStub: TunnelProviderDelegate, @unchecked Sendable {
    private let lock = NSLock()
    private var _appliedSettings: [TunnelInterfaceSettings] = []
    private var _reassertCount = 0

    let tunnelFileDescriptor: Int32?
    /// Error thrown by `applyNetworkSettings`, if any.
    var applyError: Error?

    var appliedSettings: [TunnelInterfaceSettings] { lock.withLock { _appliedSettings } }
    var reassertCount: Int { lock.withLock { _reassertCount } }

    init(tunnelFileDescriptor: Int32? = 0) {
        self.tunnelFileDescriptor = tunnelFileDescriptor
    }

    func applyNetworkSettings(_ settings: TunnelInterfaceSettings) async throws {
        if let applyError { throw applyError }
        lock.withLock { _appliedSettings.append(settings) }
    }

    func reassertTunnel() async {
        lock.withLock { _reassertCount += 1 }
    }
}

/// Factory that returns a sequence of adapters with configurable outcomes.
final class GotaTunAdapterFactoryStub: GotaTunAdapterFactory, @unchecked Sendable {
    private var outcomes: [GotaTunAdapterStub.Outcome]
    private var index = 0
    private(set) var adaptersCreated: [GotaTunAdapterStub] = []

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
        let outcome = index < outcomes.count ? outcomes[index] : outcomes[outcomes.count - 1]
        index += 1
        let adapter = GotaTunAdapterStub(outcome: outcome)
        adaptersCreated.append(adapter)
        return adapter
    }
}
