//
//  TunnelJobQueueTests.swift
//  MullvadVPNTests
//
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Operations
import XCTest

@testable import MullvadMockData
@testable import MullvadTypes
@testable import PacketTunnelCore

final class TunnelJobQueueTests: XCTestCase {
    private var tunnel: MockTunnel!
    private var interactor: MockJobQueueInteractor!
    private var operationQueue: AsyncOperationQueue!
    private var jobQueue: TunnelJobQueue!
    private let exclusivityCategory = "TunnelJobQueueTests.manageTunnel"

    override func setUp() {
        tunnel = MockTunnel(
            tunnelProvider: SimulatorTunnelProviderManager(),
            backgroundTaskProvider: UIApplicationStub()
        )
        tunnel.status = .connected

        interactor = MockJobQueueInteractor()
        interactor.tunnel = tunnel

        operationQueue = AsyncOperationQueue()

        jobQueue = TunnelJobQueue(
            interactor: interactor,
            operationQueue: operationQueue,
            backgroundTaskProvider: UIApplicationStub(),
            legacyExclusivityCategory: exclusivityCategory
        )
    }

    private func configureIPCResponder(log: EventLog, replyDelay: TimeInterval) {
        tunnel.onSendProviderMessage = { _, responseHandler in
            log.append("ipc-start")
            DispatchQueue.global().asyncAfter(deadline: .now() + replyDelay) {
                log.append("ipc-end")
                responseHandler?(try? TunnelProviderReply(ObservedState.disconnected).encode())
            }
        }
    }

    func testConcurrentReconnectsRunSerially() async throws {
        let log = EventLog()
        configureIPCResponder(log: log, replyDelay: 0.2)

        let jobQueue = self.jobQueue!
        async let firstReconnect: Void = jobQueue.reconnect(selectNewRelay: false)
        async let secondReconnect: Void = jobQueue.reconnect(selectNewRelay: true)
        _ = try await (firstReconnect, secondReconnect)

        // Strictly serial: each IPC round trip completes before the next one starts.
        XCTAssertEqual(log.events, ["ipc-start", "ipc-end", "ipc-start", "ipc-end"])
    }

    func testReconnectWaitsForLegacyExclusiveOperation() async throws {
        let log = EventLog()
        configureIPCResponder(log: log, replyDelay: 0.05)

        let legacyOperation = AsyncBlockOperation(
            dispatchQueue: nil,
            block: { finish in
                log.append("legacy-start")
                DispatchQueue.global().asyncAfter(deadline: .now() + 0.3) {
                    log.append("legacy-end")
                    finish(nil)
                }
            }
        )
        legacyOperation.addCondition(MutuallyExclusive(category: exclusivityCategory))
        operationQueue.addOperation(legacyOperation)

        try await jobQueue.reconnect(selectNewRelay: false)

        XCTAssertEqual(
            log.events,
            ["legacy-start", "legacy-end", "ipc-start", "ipc-end"],
            "Reconnect must not start while a legacy operation holds the exclusivity category"
        )
    }

    func testReconnectThrowsWhenTunnelUnset() async {
        interactor.tunnel = nil

        do {
            try await jobQueue.reconnect(selectNewRelay: false)
            XCTFail("Expected UnsetTunnelError")
        } catch {
            XCTAssertTrue(error is UnsetTunnelError, "Expected UnsetTunnelError, got \(error)")
        }
    }

    func testReconnectAppliesOptimisticStatusAndNotifies() async throws {
        let connectionState = ObservedConnectionState(
            selectedRelays: RelaySelectorStub.selectedRelays,
            relayConstraints: RelayConstraints(),
            networkReachability: .reachable,
            connectionAttemptCount: 0,
            transportLayer: .udp,
            remotePort: 1234,
            isPostQuantum: false,
            isDaitaEnabled: false
        )
        tunnel.onSendProviderMessage = { _, responseHandler in
            responseHandler?(try? TunnelProviderReply(ObservedState.connected(connectionState)).encode())
        }

        try await jobQueue.reconnect(selectNewRelay: false)

        XCTAssertTrue(interactor.didStartPolling)
        XCTAssertNil(interactor.lastReconnectError ?? nil)

        guard case .reconnecting = interactor.tunnelStatus.state else {
            return XCTFail("Expected optimistic .reconnecting state, got \(interactor.tunnelStatus.state)")
        }
    }
}

private final class MockJobQueueInteractor: TunnelJobQueueInteractor, @unchecked Sendable {
    private let lock = NSLock()

    private var _tunnel: (any TunnelProtocol)?
    private var _tunnelStatus = TunnelStatus()
    private var _didStartPolling = false
    private var _lastReconnectError: Error??

    var tunnel: (any TunnelProtocol)? {
        get {
            lock.lock()
            defer { lock.unlock() }
            return _tunnel
        }
        set {
            lock.lock()
            defer { lock.unlock() }
            _tunnel = newValue
        }
    }

    var tunnelStatus: TunnelStatus {
        lock.lock()
        defer { lock.unlock() }
        return _tunnelStatus
    }

    var didStartPolling: Bool {
        lock.lock()
        defer { lock.unlock() }
        return _didStartPolling
    }

    /// Double-optional: `nil` if `didReconnectTunnel` was never called, `.some(error)` otherwise.
    var lastReconnectError: Error?? {
        lock.lock()
        defer { lock.unlock() }
        return _lastReconnectError
    }

    func setTunnelStatus(_ block: @Sendable (inout TunnelStatus) -> Void) {
        lock.lock()
        defer { lock.unlock() }
        block(&_tunnelStatus)
    }

    func startPollingTunnelStatus() {
        lock.lock()
        defer { lock.unlock() }
        _didStartPolling = true
    }

    @MainActor
    func didReconnectTunnel(error: Error?) {
        lock.lock()
        defer { lock.unlock() }
        _lastReconnectError = .some(error)
    }
}

private final class EventLog: @unchecked Sendable {
    private let lock = NSLock()
    private var _events: [String] = []

    var events: [String] {
        lock.lock()
        defer { lock.unlock() }
        return _events
    }

    func append(_ event: String) {
        lock.lock()
        defer { lock.unlock() }
        _events.append(event)
    }
}
