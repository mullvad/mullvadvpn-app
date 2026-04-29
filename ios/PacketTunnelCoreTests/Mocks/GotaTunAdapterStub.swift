//
//  GotaTunAdapterStub.swift
//  PacketTunnelCoreTests
//
//  Created by Mullvad VPN.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import PacketTunnelCore

/// Configurable stub for `GotaTunAdapterProtocol`.
/// Schedules callbacks based on the configured outcome.
final class GotaTunAdapterStub: GotaTunAdapterProtocol, @unchecked Sendable {
    enum Outcome: Sendable {
        case connected(after: Duration = .milliseconds(10))
        case timeout(after: Duration = .milliseconds(10))
        case error(GotaTunError, after: Duration = .milliseconds(10))
        case connectedThenTimeout(
            connectedAfter: Duration = .milliseconds(10), timeoutAfter: Duration = .milliseconds(50))
    }

    let outcome: Outcome
    private(set) var lastConfig: GotaTunConfig?
    private var callbackHandler: GotaTunCallbackHandler?
    private var callbackTask: Task<Void, Never>?
    private var stopped = false

    init(outcome: Outcome = .connected()) {
        self.outcome = outcome
    }

    func startTunnel(config: GotaTunConfig, callbackHandler: GotaTunCallbackHandler) throws {
        self.lastConfig = config
        self.callbackHandler = callbackHandler
        stopped = false

        callbackTask = Task { [weak self, outcome] in
            guard let self else { return }
            switch outcome {
            case let .connected(after):
                try? await Task.sleep(for: after)
                guard !Task.isCancelled, !self.stopped else { return }
                callbackHandler.onConnected()

            case let .timeout(after):
                try? await Task.sleep(for: after)
                guard !Task.isCancelled, !self.stopped else { return }
                callbackHandler.onTimeout()

            case let .error(error, after):
                try? await Task.sleep(for: after)
                guard !Task.isCancelled, !self.stopped else { return }
                callbackHandler.onError(error)

            case let .connectedThenTimeout(connectedAfter, timeoutAfter):
                try? await Task.sleep(for: connectedAfter)
                guard !Task.isCancelled, !self.stopped else { return }
                callbackHandler.onConnected()
                try? await Task.sleep(for: timeoutAfter)
                guard !Task.isCancelled, !self.stopped else { return }
                callbackHandler.onTimeout()
            }
        }
    }

    func stopTunnel() {
        stopped = true
        callbackTask?.cancel()
        callbackTask = nil
        callbackHandler = nil
    }

    func recycleUdpSockets() {}
    func suspendTunnel() {}
    func wakeTunnel() {}
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
