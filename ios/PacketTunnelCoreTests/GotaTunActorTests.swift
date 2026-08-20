//
//  GotaTunActorTests.swift
//  PacketTunnelCoreTests
//
//  Created by Emīls on 2026-08-13.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import Network
import XCTest

@testable import MullvadMockData
@testable import MullvadREST
@testable import MullvadSettings
@preconcurrency @testable import PacketTunnelCore

final class GotaTunActorTests: XCTestCase {
    private let launchOptions = StartOptions(launchSource: .app)

    // MARK: - Helpers

    private func makeActor(
        adapterFactory: GotaTunAdapterFactory = GotaTunAdapterFactoryStub(),
        providerDelegate: TunnelProviderDelegate = TunnelProviderDelegateStub(),
        settingsReader: SettingsReaderProtocol = SettingsReaderStub.staticConfiguration(),
        relaySelector: RelaySelectorProtocol = RelaySelectorStub.nonFallible(),
        blockedStateErrorMapper: BlockedStateErrorMapperProtocol = BlockedStateErrorMapperStub()
    ) -> GotaTunActor {
        GotaTunActor(
            providerDelegate: providerDelegate,
            settingsReader: settingsReader,
            relaySelector: relaySelector,
            blockedStateErrorMapper: blockedStateErrorMapper,
            adapterFactory: adapterFactory
        )
    }

    /// Observe the actor from before `action` runs until `stop` accepts the states seen so far,
    /// returning all of them, including the one current at subscription.
    @discardableResult
    private func collectStates(
        from actor: GotaTunActor,
        timeout: TimeInterval = 2.0,
        stoppingWhen stop: @Sendable @escaping ([ObservedState]) -> Bool,
        while action: () async -> Void = {}
    ) async -> [ObservedState] {
        let collector = StateCollector()
        let reached = expectation(description: "Wait for state")
        let states = await actor.observedStates

        let task = Task {
            for await state in states {
                await collector.append(state)
                if stop(await collector.collected) {
                    reached.fulfill()
                    return
                }
            }
        }

        await action()
        await fulfillment(of: [reached], timeout: timeout)
        task.cancel()

        return collector.collected
    }

    /// Observe until a state matches `predicate`.
    @discardableResult
    private func collectStates(
        from actor: GotaTunActor,
        until predicate: @Sendable @escaping (ObservedState) -> Bool,
        timeout: TimeInterval = 2.0,
        while action: () async -> Void = {}
    ) async -> [ObservedState] {
        await collectStates(
            from: actor,
            timeout: timeout,
            stoppingWhen: { $0.last.map(predicate) ?? false },
            while: action
        )
    }

    /// Observe until `count` states have been seen.
    @discardableResult
    private func collectStates(
        from actor: GotaTunActor,
        count: Int,
        timeout: TimeInterval = 2.0,
        while action: () async -> Void = {}
    ) async -> [ObservedState] {
        await collectStates(
            from: actor,
            timeout: timeout,
            stoppingWhen: { $0.count >= count },
            while: action
        )
    }

    /// Wait for the actor to reach a state matching `predicate`, optionally driving it with `action`.
    private func waitFor(
        _ actor: GotaTunActor,
        until predicate: @Sendable @escaping (ObservedState) -> Bool,
        timeout: TimeInterval = 2.0,
        while action: () async -> Void = {}
    ) async {
        await collectStates(from: actor, until: predicate, timeout: timeout, while: action)
    }

    // MARK: - Happy path

    func testStartGoesToConnected() async throws {
        let actor = makeActor()

        await waitFor(actor, until: { $0.isConnected }, while: { await actor.start(options: launchOptions) })
    }

    func testStartTransitionsInOrder() async throws {
        let actor = makeActor()

        let states = await collectStates(
            from: actor, until: { $0.isConnected },
            while: {
                await actor.start(options: launchOptions)
            })

        XCTAssertEqual(states.map(\.name), ["Initial", "Connecting", "Connected"])
    }

    func testStartIgnoresSubsequentStarts() async throws {
        let actor = makeActor()

        await waitFor(
            actor, until: { $0.isConnected },
            while: {
                await actor.start(options: launchOptions)
                await actor.start(options: launchOptions)
            })
    }

    // MARK: - Stop

    func testStopGoesToDisconnected() async throws {
        let actor = makeActor()

        await waitFor(actor, until: { $0.isConnected }, while: { await actor.start(options: launchOptions) })

        await waitFor(actor, until: { $0.isDisconnected }, while: { actor.stop() })
    }

    // MARK: - Timeout handling

    func testTimeoutStaysInConnecting() async throws {
        let factory = GotaTunAdapterFactoryStub(outcomes: [
            .timeout(),
            .connected(),
        ])
        let actor = makeActor(adapterFactory: factory)

        let states = await collectStates(
            from: actor, until: { $0.isConnected },
            while: {
                await actor.start(options: launchOptions)
            })

        // The retry stays in `.connecting`; it must not report `.reconnecting`.
        XCTAssertEqual(states.map(\.name), ["Initial", "Connecting", "Connecting", "Connected"])
        XCTAssertEqual(factory.adaptersCreated.count, 2, "Should have created 2 adapters")
    }

    func testConnectedThenTimeoutGoesToReconnecting() async throws {
        let factory = GotaTunAdapterFactoryStub(outcomes: [
            .connectedThenTimeout(),
            .connected(),
        ])
        let actor = makeActor(adapterFactory: factory)

        let states = await collectStates(from: actor, count: 5) {
            await actor.start(options: launchOptions)
        }

        XCTAssertEqual(
            states.map(\.name),
            ["Initial", "Connecting", "Connected", "Reconnecting", "Connected"]
        )
    }

    func testAttemptCountIncrementsOnTimeout() async throws {
        let factory = GotaTunAdapterFactoryStub(outcomes: [
            .timeout(),
            .timeout(),
            .timeout(),
            .connected(),
        ])
        let actor = makeActor(adapterFactory: factory)

        await waitFor(actor, until: { $0.isConnected }, while: { await actor.start(options: launchOptions) })

        XCTAssertEqual(factory.adaptersCreated.count, 4, "Should have created 4 adapters (3 timeouts + 1 success)")
    }

    func testAttemptCountResetsOnConnected() async throws {
        let factory = GotaTunAdapterFactoryStub(outcomes: [
            .timeout(),
            .connected(),
        ])
        let actor = makeActor(adapterFactory: factory)

        let states = await collectStates(
            from: actor, until: { $0.isConnected },
            while: {
                await actor.start(options: launchOptions)
            })

        XCTAssertEqual(
            states.last?.connectionState?.connectionAttemptCount, 0, "Attempt count should reset")
    }

    // MARK: - Error state entry

    func testAdapterErrorEntersErrorState() async throws {
        let factory = GotaTunAdapterFactoryStub(outcome: .error(.internalError("test")))
        let actor = makeActor(adapterFactory: factory)

        await waitFor(
            actor, until: { $0.blockedReason != nil },
            while: {
                await actor.start(options: launchOptions)
            })
    }

    func testSettingsReadFailureEntersErrorState() async throws {
        let settingsReader = SettingsReaderStub { throw POSIXError(.EPERM) }
        let errorMapper = BlockedStateErrorMapperStub { _ in .readSettings }
        let actor = makeActor(settingsReader: settingsReader, blockedStateErrorMapper: errorMapper)

        await waitFor(
            actor, until: { $0.blockedReason == .readSettings },
            while: {
                await actor.start(options: launchOptions)
            })
    }

    func testSetErrorStateFromProvider() async throws {
        let actor = makeActor()

        await waitFor(actor, until: { $0.isConnected }, while: { await actor.start(options: launchOptions) })

        await waitFor(
            actor, until: { $0.blockedReason == .deviceRevoked },
            while: {
                actor.setErrorState(reason: .deviceRevoked)
            })
    }

    // MARK: - Error state exit

    func testUserReconnectExitsErrorState() async throws {
        let factory = GotaTunAdapterFactoryStub(outcomes: [
            .error(.internalError("test")),
            .connected(),
        ])
        let actor = makeActor(adapterFactory: factory)

        await waitFor(actor, until: { $0.blockedReason != nil }, while: { await actor.start(options: launchOptions) })

        await waitFor(
            actor, until: { $0.isConnected },
            while: {
                actor.reconnect(to: .random, reconnectReason: .userInitiated)
            })
    }

    func testConnectionLossReconnectRestartsAdapter() async throws {
        let factory = GotaTunAdapterFactoryStub(outcome: .connected())
        let actor = makeActor(adapterFactory: factory)

        await waitFor(actor, until: { $0.isConnected }, while: { await actor.start(options: launchOptions) })

        // Every reason restarts the connection; a reason that stopped the adapter without
        // starting a new one would leave the tunnel wedged with no way out.
        await collectStates(from: actor, count: 3) {
            actor.reconnect(to: .random, reconnectReason: .connectionLoss)
        }

        let state = await actor.observedState
        XCTAssertEqual(factory.adaptersCreated.count, 2)
        XCTAssertTrue(state.isConnected)
    }

    // MARK: - Stale adapter callbacks

    func testStaleConnectedCallbackIsIgnored() async throws {
        let factory = GotaTunAdapterFactoryStub(outcomes: [.connected(), .never])
        let actor = makeActor(adapterFactory: factory)

        await waitFor(actor, until: { $0.isConnected }, while: { await actor.start(options: launchOptions) })

        await waitFor(
            actor, until: { $0.isReconnecting },
            while: {
                actor.reconnect(to: .random, reconnectReason: .connectionLoss)
            })

        // A callback from the replaced adapter arrives after the swap.
        factory.adaptersCreated[0].callbackHandler?.onConnected()
        await actor.drainEvents()

        let state = await actor.observedState
        XCTAssertTrue(state.isReconnecting, "A stale onConnected must not mark the new attempt connected")
    }

    func testStaleTimeoutCallbackIsIgnored() async throws {
        let factory = GotaTunAdapterFactoryStub(outcomes: [.connected(), .never])
        let actor = makeActor(adapterFactory: factory)

        await waitFor(actor, until: { $0.isConnected }, while: { await actor.start(options: launchOptions) })

        await waitFor(
            actor, until: { $0.isReconnecting },
            while: {
                actor.reconnect(to: .random, reconnectReason: .connectionLoss)
            })

        factory.adaptersCreated[0].callbackHandler?.onTimeout()
        await actor.drainEvents()

        let state = await actor.observedState
        XCTAssertTrue(state.isReconnecting)
        XCTAssertEqual(
            factory.adaptersCreated.count, 2,
            "A stale onTimeout must not restart the current attempt")
    }

    // MARK: - Lifecycle guards

    func testReconnectBeforeStartIsIgnored() async throws {
        let factory = GotaTunAdapterFactoryStub()
        let actor = makeActor(adapterFactory: factory)

        actor.reconnect(to: .random, reconnectReason: .userInitiated)
        await actor.drainEvents()

        XCTAssertEqual(factory.adaptersCreated.count, 0, "No adapter should be created")
    }

    func testStopFromAnyState() async throws {
        let actor = makeActor()

        await waitFor(actor, until: { $0.isConnected }, while: { await actor.start(options: launchOptions) })

        actor.stop()
        await actor.drainEvents()

        let state = await actor.observedState
        XCTAssertEqual(state, .disconnected)
    }

    // MARK: - Stop during connection

    func testStopDuringConnecting() async throws {
        // The adapter never reports, so the actor stays in `.connecting` until stopped.
        let factory = GotaTunAdapterFactoryStub(outcome: .never)
        let actor = makeActor(adapterFactory: factory)

        await waitFor(actor, until: { $0.isConnecting }, while: { await actor.start(options: launchOptions) })

        actor.stop()
        await actor.drainEvents()

        let state = await actor.observedState
        XCTAssertEqual(state, .disconnected, "Should be disconnected after stop during connecting")

        XCTAssertEqual(factory.adaptersCreated.count, 1)
    }
}
