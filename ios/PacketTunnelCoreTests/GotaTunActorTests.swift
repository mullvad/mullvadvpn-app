//
//  GotaTunActorTests.swift
//  PacketTunnelCoreTests
//
//  Created by Mullvad VPN.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import Network
import XCTest

@testable import MullvadMockData
@testable import MullvadREST
@testable import MullvadSettings
@testable import PacketTunnelCore

final class GotaTunActorTests: XCTestCase {
    private let launchOptions = StartOptions(launchSource: .app)

    // MARK: - Helpers

    private func makeActor(
        adapterFactory: GotaTunAdapterFactory = GotaTunAdapterFactoryStub(),
        settingsReader: SettingsReaderProtocol = SettingsReaderStub.staticConfiguration(),
        relaySelector: RelaySelectorProtocol = RelaySelectorStub.nonFallible(),
        defaultPathObserver: DefaultPathObserverProtocol = DefaultPathObserverFake(),
        blockedStateErrorMapper: BlockedStateErrorMapperProtocol = BlockedStateErrorMapperStub(),
        timings: GotaTunActorTimings = .forTests
    ) -> GotaTunActor {
        GotaTunActor(
            timings: timings,
            tunnelFd: { 0 },  // Dummy fd for tests
            settingsReader: settingsReader,
            relaySelector: relaySelector,
            defaultPathObserver: defaultPathObserver,
            blockedStateErrorMapper: blockedStateErrorMapper,
            adapterFactory: adapterFactory
        )
    }

    /// Wait for the actor to reach a specific state, optionally performing an action first.
    private func waitForState(
        _ predicate: @Sendable @escaping (ObservedState) -> Bool,
        on actor: GotaTunActor,
        timeout: TimeInterval = 2.0,
        action: (() -> Void)? = nil
    ) async throws {
        let exp = expectation(description: "Wait for state")
        let states = await actor.observedStates

        let task = Task { @Sendable in
            for await state in states {
                if predicate(state) {
                    exp.fulfill()
                    return
                }
            }
        }

        action?()
        await fulfillment(of: [exp], timeout: timeout)
        task.cancel()
    }

    // MARK: - Happy path

    func testStartGoesToConnected() async throws {
        let actor = makeActor()

        let connectedExpectation = expectation(description: "Connected")

        let states = await actor.observedStates
        let task = Task { @Sendable in
            for await state in states {
                if case .connected = state {
                    connectedExpectation.fulfill()
                    return
                }
            }
        }

        actor.start(options: launchOptions)
        await fulfillment(of: [connectedExpectation], timeout: 2.0)
        task.cancel()
    }

    func testStartTransitionsInOrder() async throws {
        let actor = makeActor()

        let initialExpectation = expectation(description: "Initial")
        let connectingExpectation = expectation(description: "Connecting")
        let connectedExpectation = expectation(description: "Connected")

        let states = await actor.observedStates
        let task = Task { @Sendable in
            for await state in states {
                switch state {
                case .initial:
                    initialExpectation.fulfill()
                case .connecting:
                    connectingExpectation.fulfill()
                case .connected:
                    connectedExpectation.fulfill()
                    return
                default:
                    break
                }
            }
        }

        actor.start(options: launchOptions)
        await fulfillment(
            of: [initialExpectation, connectingExpectation, connectedExpectation],
            timeout: 2.0,
            enforceOrder: true
        )
        task.cancel()
    }

    func testStartIgnoresSubsequentStarts() async throws {
        let actor = makeActor()

        let connectedExpectation = expectation(description: "Connected")

        let states = await actor.observedStates
        let task = Task { @Sendable in
            for await state in states {
                if case .connected = state {
                    connectedExpectation.fulfill()
                    return
                }
            }
        }

        actor.start(options: launchOptions)
        actor.start(options: launchOptions)  // Should be ignored
        await fulfillment(of: [connectedExpectation], timeout: 2.0)
        task.cancel()
    }

    // MARK: - Stop

    func testStopGoesToDisconnected() async throws {
        let actor = makeActor()
        let options = launchOptions

        // First connect
        try await waitForState(
            {
                if case .connected = $0 { return true }
                return false
            }, on: actor
        ) {
            actor.start(options: options)
        }

        // Then stop
        try await waitForState(
            {
                if case .disconnected = $0 { return true }
                return false
            }, on: actor
        ) {
            actor.stop()
        }
    }

    func testStopIsNoopBeforeStart() async throws {
        let actor = makeActor()
        actor.stop()  // Should not crash or change state

        let state = await actor.observedState
        // State could be initial or disconnected depending on timing
        // The important thing is it doesn't crash
        XCTAssertTrue(state == .initial || state == .disconnected)
    }

    // MARK: - Timeout handling

    func testTimeoutStaysInConnecting() async throws {
        // First attempt times out, second succeeds
        let factory = GotaTunAdapterFactoryStub(outcomes: [
            .timeout(),
            .connected(),
        ])
        let actor = makeActor(adapterFactory: factory)

        let connectingExpectation = expectation(description: "Connecting")
        connectingExpectation.assertForOverFulfill = false
        let connectedExpectation = expectation(description: "Connected after retry")

        let states = await actor.observedStates
        let task = Task { @Sendable in
            for await state in states {
                if case .connecting = state {
                    connectingExpectation.fulfill()
                }
                if case .connected = state {
                    connectedExpectation.fulfill()
                    return
                }
            }
        }

        actor.start(options: launchOptions)
        await fulfillment(of: [connectingExpectation, connectedExpectation], timeout: 2.0)
        task.cancel()

        XCTAssertEqual(factory.adaptersCreated.count, 2, "Should have created 2 adapters")
    }

    func testConnectedThenTimeoutGoesToReconnecting() async throws {
        let factory = GotaTunAdapterFactoryStub(outcomes: [
            .connectedThenTimeout(),
            .connected(),
        ])
        let actor = makeActor(adapterFactory: factory)

        let connectedExpectation = expectation(description: "Connected first time")
        let reconnectingExpectation = expectation(description: "Reconnecting")
        reconnectingExpectation.assertForOverFulfill = false
        let reconnectedExpectation = expectation(description: "Reconnected")

        let states = await actor.observedStates
        let task = Task { @Sendable in
            var connectedOnce = false
            for await state in states {
                switch state {
                case .connected:
                    if connectedOnce {
                        reconnectedExpectation.fulfill()
                        return
                    }
                    connectedOnce = true
                    connectedExpectation.fulfill()
                case .reconnecting:
                    reconnectingExpectation.fulfill()
                default:
                    break
                }
            }
        }

        actor.start(options: launchOptions)
        await fulfillment(
            of: [connectedExpectation, reconnectingExpectation, reconnectedExpectation], timeout: 2.0,
            enforceOrder: true)
        task.cancel()
    }

    func testAttemptCountIncrementsOnTimeout() async throws {
        // 3 timeouts, then success
        let factory = GotaTunAdapterFactoryStub(outcomes: [
            .timeout(),
            .timeout(),
            .timeout(),
            .connected(),
        ])
        let actor = makeActor(adapterFactory: factory)

        let connectedExpectation = expectation(description: "Connected")

        let states = await actor.observedStates
        let task = Task { @Sendable in
            for await state in states {
                if case .connected = state {
                    connectedExpectation.fulfill()
                    return
                }
            }
        }

        actor.start(options: launchOptions)
        await fulfillment(of: [connectedExpectation], timeout: 2.0)
        task.cancel()

        // Verify multiple adapters were created (one per attempt)
        XCTAssertEqual(factory.adaptersCreated.count, 4, "Should have created 4 adapters (3 timeouts + 1 success)")
    }

    func testAttemptCountResetsOnConnected() async throws {
        let factory = GotaTunAdapterFactoryStub(outcomes: [
            .timeout(),
            .connected(),
        ])
        let actor = makeActor(adapterFactory: factory)

        let connectedExpectation = expectation(description: "Connected")

        let states = await actor.observedStates
        let task = Task { @Sendable in
            for await state in states {
                if case let .connected(connectionState) = state {
                    XCTAssertEqual(connectionState.connectionAttemptCount, 0, "Attempt count should reset")
                    connectedExpectation.fulfill()
                    return
                }
            }
        }

        actor.start(options: launchOptions)
        await fulfillment(of: [connectedExpectation], timeout: 2.0)
        task.cancel()
    }

    // MARK: - Error state entry

    func testAdapterErrorEntersErrorState() async throws {
        let factory = GotaTunAdapterFactoryStub(outcome: .error(.internalError("test")))
        let actor = makeActor(adapterFactory: factory)

        let errorExpectation = expectation(description: "Error state")

        let states = await actor.observedStates
        let task = Task { @Sendable in
            for await state in states {
                if case .error = state {
                    errorExpectation.fulfill()
                    return
                }
            }
        }

        actor.start(options: launchOptions)
        await fulfillment(of: [errorExpectation], timeout: 2.0)
        task.cancel()
    }

    func testSettingsReadFailureEntersErrorState() async throws {
        let settingsReader = SettingsReaderStub { throw POSIXError(.EPERM) }
        let errorMapper = BlockedStateErrorMapperStub { _ in .readSettings }
        let actor = makeActor(settingsReader: settingsReader, blockedStateErrorMapper: errorMapper)

        let errorExpectation = expectation(description: "Error state")

        let states = await actor.observedStates
        let task = Task { @Sendable in
            for await state in states {
                if case let .error(blocked) = state {
                    XCTAssertEqual(blocked.reason, .readSettings)
                    errorExpectation.fulfill()
                    return
                }
            }
        }

        actor.start(options: launchOptions)
        await fulfillment(of: [errorExpectation], timeout: 2.0)
        task.cancel()
    }

    func testSetErrorStateFromProvider() async throws {
        let actor = makeActor()
        let options = launchOptions

        // Start and connect first
        try await waitForState(
            {
                if case .connected = $0 { return true }
                return false
            }, on: actor
        ) {
            actor.start(options: options)
        }

        // Provider sets error (e.g. device revoked)
        let errorExpectation = expectation(description: "Error state")

        let states = await actor.observedStates
        let task = Task { @Sendable in
            for await state in states {
                if case let .error(blocked) = state {
                    XCTAssertEqual(blocked.reason, .deviceRevoked)
                    errorExpectation.fulfill()
                    return
                }
            }
        }

        actor.setErrorState(reason: .deviceRevoked)
        await fulfillment(of: [errorExpectation], timeout: 2.0)
        task.cancel()
    }

    // MARK: - Error state exit

    func testUserReconnectExitsErrorState() async throws {
        let factory = GotaTunAdapterFactoryStub(outcomes: [
            .error(.internalError("test")),
            .connected(),
        ])
        let actor = makeActor(adapterFactory: factory)

        let options = launchOptions

        // Start → error
        try await waitForState(
            {
                if case .error = $0 { return true }
                return false
            }, on: actor
        ) {
            actor.start(options: options)
        }

        // Reconnect → should exit error
        let connectedExpectation = expectation(description: "Connected after reconnect")

        let states = await actor.observedStates
        let task = Task { @Sendable in
            for await state in states {
                if case .connected = state {
                    connectedExpectation.fulfill()
                    return
                }
            }
        }

        actor.reconnect(to: .random, reconnectReason: .userInitiated)
        await fulfillment(of: [connectedExpectation], timeout: 2.0)
        task.cancel()
    }

    // MARK: - Lifecycle guards

    func testReconnectBeforeStartIsIgnored() async throws {
        let factory = GotaTunAdapterFactoryStub()
        let actor = makeActor(adapterFactory: factory)

        actor.reconnect(to: .random, reconnectReason: .userInitiated)

        // Give it time to process
        try await Task.sleep(for: .milliseconds(50))

        XCTAssertEqual(factory.adaptersCreated.count, 0, "No adapter should be created")
    }

    func testStopFromAnyState() async throws {
        let actor = makeActor()
        let options = launchOptions

        // Start and connect
        try await waitForState(
            {
                if case .connected = $0 { return true }
                return false
            }, on: actor
        ) {
            actor.start(options: options)
        }

        // Stop
        actor.stop()
        try await Task.sleep(for: .milliseconds(50))

        let state = await actor.observedState
        XCTAssertEqual(state, .disconnected)
    }

    // MARK: - Network reachability

    func testOfflineEntersErrorState() async throws {
        let actor = makeActor()
        let options = launchOptions

        // Start and connect
        try await waitForState(
            {
                if case .connected = $0 { return true }
                return false
            }, on: actor
        ) {
            actor.start(options: options)
        }

        // Go offline
        let errorExpectation = expectation(description: "Offline error")

        let states = await actor.observedStates
        let task = Task { @Sendable in
            for await state in states {
                if case let .error(blocked) = state, blocked.reason == .offline {
                    errorExpectation.fulfill()
                    return
                }
            }
        }

        actor.updateNetworkReachability(networkPathStatus: .unsatisfied)
        await fulfillment(of: [errorExpectation], timeout: 2.0)
        task.cancel()
    }

    func testOnlineAfterOfflineReconnects() async throws {
        let factory = GotaTunAdapterFactoryStub(outcome: .connected())
        let actor = makeActor(adapterFactory: factory)
        let options = launchOptions

        // Start and connect
        try await waitForState(
            {
                if case .connected = $0 { return true }
                return false
            }, on: actor
        ) {
            actor.start(options: options)
        }

        // Go offline
        try await waitForState(
            {
                if case let .error(b) = $0, b.reason == .offline { return true }
                return false
            }, on: actor
        ) {
            actor.updateNetworkReachability(networkPathStatus: .unsatisfied)
        }

        // Go online — should reconnect
        let reconnectedExpectation = expectation(description: "Reconnected")

        let states = await actor.observedStates
        let task = Task { @Sendable in
            for await state in states {
                if case .connected = state {
                    reconnectedExpectation.fulfill()
                    return
                }
            }
        }

        actor.updateNetworkReachability(networkPathStatus: .satisfied)
        await fulfillment(of: [reconnectedExpectation], timeout: 2.0)
        task.cancel()
    }
}

// MARK: - Test timings

extension GotaTunActorTimings {
    static var forTests: GotaTunActorTimings {
        GotaTunActorTimings(
            bootRecoveryPeriodicity: .milliseconds(10),
            wgKeyPropagationDelay: .zero
        )
    }
}
