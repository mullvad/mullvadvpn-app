// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
        defaultPathObserver: GotaTunPathObserverFake = GotaTunPathObserverFake(),
        blockedStateErrorMapper: BlockedStateErrorMapperProtocol = BlockedStateErrorMapperStub(),
        clock: TestClock = TestClock(),
        timings: GotaTunActorTimings = GotaTunActorTimings()
    ) -> GotaTunActor {
        GotaTunActor(
            timings: timings,
            clock: clock,
            providerDelegate: providerDelegate,
            settingsReader: settingsReader,
            relaySelector: relaySelector,
            defaultPathObserver: defaultPathObserver,
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

        return await collector.collected
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

    /// Reach `.reconnecting` on a second adapter, leaving `adaptersCreated[0]` replaced but still
    /// able to report, the way a Rust device can when its callback is already in flight.
    private func makeActorWithReplacedAdapter() async -> (GotaTunActor, GotaTunAdapterFactoryStub) {
        let factory = GotaTunAdapterFactoryStub(outcomes: [.connected(), .never])
        let actor = makeActor(adapterFactory: factory)

        await waitFor(actor, until: { $0.isConnected }, while: { await actor.start(options: launchOptions) })

        await waitFor(
            actor, until: { $0.isReconnecting },
            while: {
                actor.reconnect(to: .random, reconnectReason: .connectionLoss)
            })

        return (actor, factory)
    }

    /// Stop the actor and assert it settles in `.disconnected`.
    private func assertStopLeadsToDisconnected(
        _ actor: GotaTunActor,
        file: StaticString = #filePath,
        line: UInt = #line
    ) async {
        await waitFor(actor, until: { $0.isDisconnected }, while: { actor.stop() })

        let state = await actor.observedState
        XCTAssertEqual(state, .disconnected, file: file, line: line)
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

    func testStopIsNoopBeforeStart() async throws {
        let pathObserver = GotaTunPathObserverFake()
        let actor = makeActor(defaultPathObserver: pathObserver)
        actor.stop()
        await actor.drainEvents()

        let state = await actor.observedState
        XCTAssertEqual(state, .disconnected)
        let stopCount = await pathObserver.stopCount
        XCTAssertEqual(stopCount, 1, "Stopping must tear observation down")
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

    // MARK: - Callbacks from unexpected adapters

    func testStaleConnectedCallbackIsIgnored() async throws {
        let (actor, factory) = await makeActorWithReplacedAdapter()

        // A callback from the replaced adapter arrives after the swap.
        factory.adaptersCreated[0].callbackHandler?.onConnected()
        await actor.drainEvents()

        let state = await actor.observedState
        XCTAssertTrue(state.isReconnecting, "A stale onConnected must not mark the new attempt connected")
    }

    func testStaleTimeoutCallbackIsIgnored() async throws {
        let (actor, factory) = await makeActorWithReplacedAdapter()

        factory.adaptersCreated[0].callbackHandler?.onTimeout()
        await actor.drainEvents()

        let state = await actor.observedState
        XCTAssertTrue(state.isReconnecting)
        XCTAssertEqual(
            factory.adaptersCreated.count, 2,
            "A stale onTimeout must not restart the current attempt")
    }

    func testStaleErrorCallbackIsIgnored() async throws {
        let (actor, factory) = await makeActorWithReplacedAdapter()

        factory.adaptersCreated[0].callbackHandler?.onError(.internalError("stale"))
        await actor.drainEvents()

        let state = await actor.observedState
        XCTAssertNil(state.blockedReason, "A stale onError must not block the tunnel")
        XCTAssertTrue(state.isReconnecting)
    }

    /// A stale adapter that reports repeatedly must stay ignored, not just on its first callback.
    func testRepeatedStaleCallbacksAreIgnored() async throws {
        let (actor, factory) = await makeActorWithReplacedAdapter()

        let stale = factory.adaptersCreated[0]
        stale.callbackHandler?.onConnected()
        stale.callbackHandler?.onTimeout()
        stale.callbackHandler?.onConnected()
        stale.callbackHandler?.onError(.invalidConfig("stale"))
        await actor.drainEvents()

        let state = await actor.observedState
        XCTAssertTrue(state.isReconnecting)
        XCTAssertEqual(factory.adaptersCreated.count, 2)
    }

    /// Entering the error state stops the adapter without replacing it. Its callbacks must not
    /// resurrect the connection, nor rewrite the reason the tunnel is blocked for.
    func testCallbacksFromAdapterStoppedByErrorStateAreIgnored() async throws {
        let factory = GotaTunAdapterFactoryStub(outcome: .never)
        let actor = makeActor(adapterFactory: factory)

        await waitFor(actor, until: { $0.isConnecting }, while: { await actor.start(options: launchOptions) })

        await waitFor(
            actor, until: { $0.blockedReason == .deviceRevoked },
            while: {
                actor.setErrorState(reason: .deviceRevoked)
            })

        let stopped = factory.adaptersCreated[0]
        XCTAssertTrue(stopped.isStopped)

        stopped.callbackHandler?.onConnected()
        stopped.callbackHandler?.onError(.internalError("late"))
        stopped.callbackHandler?.onTimeout()
        await actor.drainEvents()

        let state = await actor.observedState
        XCTAssertEqual(state.blockedReason, .deviceRevoked, "A stopped adapter must not change the blocked reason")
        XCTAssertEqual(factory.adaptersCreated.count, 1, "A stopped adapter must not trigger a new attempt")
    }

    func testCallbacksAfterStopAreIgnored() async throws {
        let factory = GotaTunAdapterFactoryStub(outcome: .never)
        let actor = makeActor(adapterFactory: factory)

        await waitFor(actor, until: { $0.isConnecting }, while: { await actor.start(options: launchOptions) })
        await assertStopLeadsToDisconnected(actor)

        let stopped = factory.adaptersCreated[0]
        stopped.callbackHandler?.onConnected()
        stopped.callbackHandler?.onTimeout()
        stopped.callbackHandler?.onError(.internalError("late"))
        await actor.drainEvents()

        let state = await actor.observedState
        XCTAssertEqual(state, .disconnected, "Nothing brings the actor back out of `.disconnected`")
        XCTAssertEqual(factory.adaptersCreated.count, 1)
    }

    // MARK: - Lifecycle guards

    func testReconnectBeforeStartIsIgnored() async throws {
        let factory = GotaTunAdapterFactoryStub()
        let actor = makeActor(adapterFactory: factory)

        actor.reconnect(to: .random, reconnectReason: .userInitiated)
        await actor.drainEvents()

        XCTAssertEqual(factory.adaptersCreated.count, 0, "No adapter should be created")
    }

    // MARK: - Stop from every state

    // `.negotiatingEphemeralPeer` and `.disconnecting` are unreachable for this actor: PQ is
    // handled inside Rust, and stopping enters `.disconnected` directly.

    func testStopFromInitial() async throws {
        let factory = GotaTunAdapterFactoryStub()
        let actor = makeActor(adapterFactory: factory)

        await assertStopLeadsToDisconnected(actor)

        XCTAssertEqual(factory.adaptersCreated.count, 0, "Stopping before start must not connect anything")
    }

    // MARK: - Network reachability

    func testOfflineEntersErrorState() async throws {
        let pathObserver = GotaTunPathObserverFake()
        let actor = makeActor(defaultPathObserver: pathObserver)

        await waitFor(actor, until: { $0.isConnected }, while: { await actor.start(options: launchOptions) })

        await waitFor(
            actor, until: { $0.blockedReason == .offline },
            while: {
                await pathObserver.updatePath(.unsatisfied)
            })
    }

    func testOfflinePathUpdateWhileConnectedBlocksAndStopsAdapter() async throws {
        let factory = GotaTunAdapterFactoryStub(outcome: .connected())
        let pathObserver = GotaTunPathObserverFake()
        let actor = makeActor(adapterFactory: factory, defaultPathObserver: pathObserver)

        await waitFor(actor, until: { $0.isConnected }, while: { await actor.start(options: launchOptions) })
        XCTAssertEqual(factory.adaptersCreated.count, 1)
        XCTAssertFalse(factory.adaptersCreated[0].isStopped)

        await waitFor(
            actor, until: { $0.blockedReason == .offline },
            while: {
                await pathObserver.updatePath(.unsatisfied)
            })
        await actor.drainEvents()

        XCTAssertTrue(
            factory.adaptersCreated[0].isStopped,
            "The tunnel must be torn down when the path is lost")
        XCTAssertEqual(
            factory.adaptersCreated.count, 1,
            "Going offline must block rather than start another attempt")
    }

    func testPathObserverStartsWithTheActor() async throws {
        let pathObserver = GotaTunPathObserverFake()
        let actor = makeActor(defaultPathObserver: pathObserver)

        await actor.drainEvents()
        var startCount = await pathObserver.startCount
        XCTAssertEqual(startCount, 1)

        await waitFor(actor, until: { $0.isConnected }, while: { await actor.start(options: launchOptions) })

        startCount = await pathObserver.startCount
        XCTAssertEqual(startCount, 1, "Starting the tunnel must not restart observation")
    }

    func testStartedWhileOfflineBlocksImmediately() async throws {
        let factory = GotaTunAdapterFactoryStub(outcome: .connected())
        let pathObserver = GotaTunPathObserverFake(status: .unsatisfied)
        let actor = makeActor(adapterFactory: factory, defaultPathObserver: pathObserver)

        await waitFor(
            actor, until: { $0.blockedReason == .offline }, while: { await actor.start(options: launchOptions) })

        XCTAssertEqual(factory.adaptersCreated.count, 0, "Must not attempt to connect while offline")
    }

    func testStartedWhileOfflineThenOnlineConnects() async throws {
        let factory = GotaTunAdapterFactoryStub(outcome: .connected())
        let pathObserver = GotaTunPathObserverFake(status: .unsatisfied)
        let actor = makeActor(adapterFactory: factory, defaultPathObserver: pathObserver)

        await waitFor(
            actor, until: { $0.blockedReason == .offline }, while: { await actor.start(options: launchOptions) })

        await waitFor(actor, until: { $0.isConnected }, while: { await pathObserver.updatePath(.satisfied) })

        XCTAssertEqual(factory.adaptersCreated.count, 1)
    }

    func testPathObserverStoppedOnTeardown() async throws {
        let pathObserver = GotaTunPathObserverFake()
        let actor = makeActor(defaultPathObserver: pathObserver)

        await waitFor(actor, until: { $0.isConnected }, while: { await actor.start(options: launchOptions) })

        await waitFor(actor, until: { $0.isDisconnected }, while: { actor.stop() })

        let stopCount = await pathObserver.stopCount
        XCTAssertEqual(stopCount, 1)
        let isStarted = await pathObserver.isStarted
        XCTAssertFalse(isStarted)
    }

    func testSatisfiedPathRecyclesSockets() async throws {
        let factory = GotaTunAdapterFactoryStub(outcome: .connected())
        let pathObserver = GotaTunPathObserverFake()
        let actor = makeActor(adapterFactory: factory, defaultPathObserver: pathObserver)

        await waitFor(actor, until: { $0.isConnected }, while: { await actor.start(options: launchOptions) })

        let recycled = expectation(description: "Sockets recycled")
        factory.adaptersCreated[0].onRecycleUdpSockets = { recycled.fulfill() }

        await pathObserver.updatePath(.satisfied)
        await fulfillment(of: [recycled], timeout: 2.0)

        XCTAssertEqual(factory.adaptersCreated[0].recycleUdpSocketsCount, 1)
        XCTAssertEqual(factory.adaptersCreated.count, 1)

        let state = await actor.observedState
        XCTAssertTrue(state.isConnected, "Expected to remain connected, got \(state.name)")
    }

    func testRequiresConnectionDoesNotGoOffline() async throws {
        let factory = GotaTunAdapterFactoryStub(outcome: .connected())
        let pathObserver = GotaTunPathObserverFake()
        let actor = makeActor(adapterFactory: factory, defaultPathObserver: pathObserver)

        await waitFor(actor, until: { $0.isConnected }, while: { await actor.start(options: launchOptions) })

        await pathObserver.updatePath(.requiresConnection)
        await actor.drainEvents()

        let state = await actor.observedState
        XCTAssertTrue(state.isConnected, "Expected to remain connected, got \(state.name)")
        XCTAssertEqual(state.connectionState?.networkReachability, .reachable)
    }

    func testRepeatedUnsatisfiedDoesNotChurn() async throws {
        let factory = GotaTunAdapterFactoryStub(outcome: .connected())
        let pathObserver = GotaTunPathObserverFake()
        let actor = makeActor(adapterFactory: factory, defaultPathObserver: pathObserver)

        await waitFor(actor, until: { $0.isConnected }, while: { await actor.start(options: launchOptions) })

        let offlineExpectation = expectation(description: "Offline error emitted exactly once")
        offlineExpectation.assertForOverFulfill = true

        let states = await actor.observedStates
        let task = Task {
            for await state in states where state.blockedReason == .offline {
                offlineExpectation.fulfill()
            }
        }

        await pathObserver.updatePath(.unsatisfied)
        await pathObserver.updatePath(.unsatisfied)
        await pathObserver.updatePath(.unsatisfied)
        await fulfillment(of: [offlineExpectation], timeout: 2.0)
        await actor.drainEvents()
        task.cancel()

        XCTAssertEqual(factory.adaptersCreated.count, 1)
    }

    func testObservedReachabilityTracksPath() async throws {
        let pathObserver = GotaTunPathObserverFake()
        let actor = makeActor(defaultPathObserver: pathObserver)

        await waitFor(
            actor,
            until: { $0.isConnecting && $0.connectionState?.networkReachability == .reachable },
            while: { await actor.start(options: launchOptions) }
        )
    }

    func testOnlineAfterOfflineReconnects() async throws {
        let factory = GotaTunAdapterFactoryStub(outcome: .connected())
        let pathObserver = GotaTunPathObserverFake()
        let actor = makeActor(adapterFactory: factory, defaultPathObserver: pathObserver)

        await waitFor(actor, until: { $0.isConnected }, while: { await actor.start(options: launchOptions) })

        await waitFor(
            actor, until: { $0.blockedReason == .offline }, while: { await pathObserver.updatePath(.unsatisfied) })

        await waitFor(actor, until: { $0.isConnected }, while: { await pathObserver.updatePath(.satisfied) })

        XCTAssertEqual(factory.adaptersCreated.count, 2)
    }

    // MARK: - Key rotation

    func testKeyRotationTriggersReconnect() async throws {
        let factory = GotaTunAdapterFactoryStub(outcome: .connected())
        let clock = TestClock()
        let timings = GotaTunActorTimings()
        let actor = makeActor(adapterFactory: factory, clock: clock, timings: timings)

        await waitFor(actor, until: { $0.isConnected }, while: { await actor.start(options: launchOptions) })

        XCTAssertEqual(factory.adaptersCreated.count, 1)

        actor.notifyKeyRotation(date: Date())
        await actor.drainEvents()
        XCTAssertEqual(factory.adaptersCreated.count, 1, "Must not reconnect before the key has propagated")

        await clock.waitForSleepers()
        let states = await collectStates(from: actor, count: 3) {
            Task { await clock.advance(by: timings.wgKeyPropagationDelay) }
        }

        XCTAssertEqual(states.map(\.name), ["Connected", "Reconnecting", "Connected"])
        XCTAssertEqual(factory.adaptersCreated.count, 2, "Should have created a new adapter for key rotation")
    }

    func testKeyRotationKeepsPriorKeyUntilPropagated() async throws {
        let priorKey = WireGuard.PrivateKey()
        let rotatedKey = WireGuard.PrivateKey()
        // The app persists the new key before notifying the tunnel, so a read after the
        // notification already returns the rotated key.
        var hasRotated = false
        let settingsReader = SettingsReaderStub {
            Settings(
                privateKey: hasRotated ? rotatedKey : priorKey,
                interfaceAddresses: [
                    IPAddressRange(from: "127.0.0.1/32")!,
                    IPAddressRange(from: "fc00::1/128")!,
                ],
                tunnelSettings: LatestTunnelSettings()
            )
        }
        let factory = GotaTunAdapterFactoryStub(outcome: .connected())
        let clock = TestClock()
        let timings = GotaTunActorTimings()
        let actor = makeActor(
            adapterFactory: factory,
            settingsReader: settingsReader,
            clock: clock,
            timings: timings
        )

        await waitFor(actor, until: { $0.isConnected }, while: { await actor.start(options: launchOptions) })
        XCTAssertEqual(factory.adaptersCreated[0].lastConfig?.privateKey, priorKey.rawValue)

        hasRotated = true
        actor.notifyKeyRotation(date: Date())
        await actor.drainEvents()

        await collectStates(from: actor, count: 3) {
            actor.reconnect(to: .random, reconnectReason: .userInitiated)
        }
        XCTAssertEqual(
            factory.adaptersCreated[1].lastConfig?.privateKey, priorKey.rawValue,
            "Reconnects during the propagation window must keep using the prior key")
        XCTAssertEqual(
            factory.adaptersCreated[1].lastConfig?.clientPublicKey, priorKey.publicKey.rawValue,
            "The obfuscator's client public key must match the connection key, not the settings key")

        await clock.waitForSleepers()
        await collectStates(from: actor, count: 3) {
            Task { await clock.advance(by: timings.wgKeyPropagationDelay) }
        }
        XCTAssertEqual(factory.adaptersCreated[2].lastConfig?.privateKey, rotatedKey.rawValue)
    }

    func testErrorStateDuringKeyRotationEndsRotation() async throws {
        let priorKey = WireGuard.PrivateKey()
        let rotatedKey = WireGuard.PrivateKey()
        var hasRotated = false
        let settingsReader = SettingsReaderStub {
            Settings(
                privateKey: hasRotated ? rotatedKey : priorKey,
                interfaceAddresses: [
                    IPAddressRange(from: "127.0.0.1/32")!,
                    IPAddressRange(from: "fc00::1/128")!,
                ],
                tunnelSettings: LatestTunnelSettings()
            )
        }
        let factory = GotaTunAdapterFactoryStub(outcome: .connected())
        let pathObserver = GotaTunPathObserverFake()
        let actor = makeActor(
            adapterFactory: factory,
            settingsReader: settingsReader,
            defaultPathObserver: pathObserver
        )

        await waitFor(actor, until: { $0.isConnected }, while: { await actor.start(options: launchOptions) })

        hasRotated = true
        actor.notifyKeyRotation(date: Date())
        await actor.drainEvents()

        await waitFor(
            actor, until: { $0.blockedReason == .offline }, while: { await pathObserver.updatePath(.unsatisfied) })

        await waitFor(actor, until: { $0.isConnected }, while: { await pathObserver.updatePath(.satisfied) })
        XCTAssertEqual(
            factory.adaptersCreated[1].lastConfig?.privateKey, rotatedKey.rawValue,
            "Reconnecting after an error state must use the rotated key, not the stale prior key")
    }

    func testKeyRotationDatePropagatesToObservedState() async throws {
        let actor = makeActor()
        let rotationDate = Date()

        await waitFor(actor, until: { $0.isConnected }, while: { await actor.start(options: launchOptions) })
        let beforeRotation = await actor.observedState
        XCTAssertNil(beforeRotation.connectionState?.lastKeyRotation)

        actor.notifyKeyRotation(date: rotationDate)
        await actor.drainEvents()

        // The app compares this against its own record to decide whether to reload device state.
        let afterRotation = await actor.observedState
        XCTAssertEqual(afterRotation.connectionState?.lastKeyRotation, rotationDate)
    }

    // MARK: - Cascading errors

    func testCascadingErrorsWithRecovery() async throws {
        // Settings unreadable for the first two attempts (e.g. device locked at boot), then readable.
        // Should retry automatically via the boot recovery timer, with no adapter created until settings
        // are actually readable.
        var readAttempts = 0
        let settingsReader = SettingsReaderStub {
            readAttempts += 1
            guard readAttempts > 2 else { throw POSIXError(.EPERM) }
            return Settings(
                privateKey: WireGuard.PrivateKey(),
                interfaceAddresses: [
                    IPAddressRange(from: "127.0.0.1/32")!,
                    IPAddressRange(from: "fc00::1/128")!,
                ],
                tunnelSettings: LatestTunnelSettings()
            )
        }
        let errorMapper = BlockedStateErrorMapperStub { _ in .deviceLocked }
        let factory = GotaTunAdapterFactoryStub(outcome: .connected())
        let clock = TestClock()
        let timings = GotaTunActorTimings()
        let actor = makeActor(
            adapterFactory: factory,
            settingsReader: settingsReader,
            blockedStateErrorMapper: errorMapper,
            clock: clock,
            timings: timings
        )

        await waitFor(
            actor, until: { $0.blockedReason == .deviceLocked }, while: { await actor.start(options: launchOptions) })
        await actor.drainEvents()
        XCTAssertEqual(readAttempts, 1)

        await clock.waitForSleepers()
        await clock.advance(by: timings.bootRecoveryPeriodicity)
        await actor.drainEvents()
        XCTAssertEqual(readAttempts, 2)
        XCTAssertEqual(factory.adaptersCreated.count, 0, "No adapter while settings are unreadable")

        await clock.waitForSleepers()
        await waitFor(
            actor, until: { $0.isConnected },
            while: {
                Task { await clock.advance(by: timings.bootRecoveryPeriodicity) }
            })

        XCTAssertEqual(readAttempts, 3, "Should have retried settings read until it succeeded")
        XCTAssertEqual(factory.adaptersCreated.count, 1, "Adapter should only be created once settings are readable")
    }

    func testNonAutoRestartableErrorDoesNotRetry() async throws {
        var readAttempts = 0
        let settingsReader = SettingsReaderStub {
            readAttempts += 1
            throw POSIXError(.EPERM)
        }
        let errorMapper = BlockedStateErrorMapperStub { _ in .readSettings }
        let factory = GotaTunAdapterFactoryStub(outcome: .connected())
        let clock = TestClock()
        let timings = GotaTunActorTimings()
        let actor = makeActor(
            adapterFactory: factory,
            settingsReader: settingsReader,
            blockedStateErrorMapper: errorMapper,
            clock: clock,
            timings: timings
        )

        await waitFor(
            actor, until: { $0.blockedReason == .readSettings }, while: { await actor.start(options: launchOptions) })

        await actor.drainEvents()
        XCTAssertEqual(clock.sleeperCount, 0, "No recovery timer should be armed")

        // Span many recovery periods instantly; nothing should be scheduled to act on them.
        await clock.advance(by: timings.bootRecoveryPeriodicity * 20)
        await actor.drainEvents()

        XCTAssertEqual(readAttempts, 1, "readSettings is not auto-restartable and must not be retried")
        XCTAssertEqual(factory.adaptersCreated.count, 0)
    }

    func testRecoveryStopsWhenReasonBecomesNonRestartable() async throws {
        var readAttempts = 0
        let settingsReader = SettingsReaderStub {
            readAttempts += 1
            throw POSIXError(.EPERM)
        }
        let errorMapper = BlockedStateErrorMapperStub { _ in .deviceLocked }
        let factory = GotaTunAdapterFactoryStub(outcome: .connected())
        let clock = TestClock()
        let timings = GotaTunActorTimings()
        let actor = makeActor(
            adapterFactory: factory,
            settingsReader: settingsReader,
            blockedStateErrorMapper: errorMapper,
            clock: clock,
            timings: timings
        )

        await waitFor(
            actor, until: { $0.blockedReason == .deviceLocked }, while: { await actor.start(options: launchOptions) })
        await actor.drainEvents()
        XCTAssertEqual(readAttempts, 1)

        // Each period wakes the recovery timer exactly once. The timer is replaced on every
        // cycle, so wait for the new one to arm before advancing again.
        for expectedAttempts in 2...4 {
            await clock.waitForSleepers()
            await clock.advance(by: timings.bootRecoveryPeriodicity)
            await actor.drainEvents()
            XCTAssertEqual(readAttempts, expectedAttempts, "deviceLocked should be retried once per recovery period")
        }

        await waitFor(
            actor, until: { $0.blockedReason == .deviceRevoked },
            while: {
                actor.setErrorState(reason: .deviceRevoked)
            })

        await actor.drainEvents()
        XCTAssertEqual(clock.sleeperCount, 0, "Recovery timer must be cancelled")

        await clock.advance(by: timings.bootRecoveryPeriodicity * 20)
        await actor.drainEvents()

        XCTAssertEqual(readAttempts, 4, "Recovery must stop once the reason is not restartable")
    }

    // MARK: - Stop during connection

    func testStopDuringConnecting() async throws {
        // The adapter never reports, so the actor stays in `.connecting` until stopped.
        let factory = GotaTunAdapterFactoryStub(outcome: .never)
        let actor = makeActor(adapterFactory: factory)

        await waitFor(actor, until: { $0.isConnecting }, while: { await actor.start(options: launchOptions) })

        await assertStopLeadsToDisconnected(actor)

        XCTAssertEqual(factory.adaptersCreated.count, 1)
        XCTAssertTrue(factory.adaptersCreated[0].isStopped, "The in-flight attempt must be torn down")
    }

    func testStopFromConnected() async throws {
        let factory = GotaTunAdapterFactoryStub()
        let actor = makeActor(adapterFactory: factory)

        await waitFor(actor, until: { $0.isConnected }, while: { await actor.start(options: launchOptions) })

        await assertStopLeadsToDisconnected(actor)

        XCTAssertTrue(factory.adaptersCreated[0].isStopped)
    }

    func testStopFromReconnecting() async throws {
        let (actor, factory) = await makeActorWithReplacedAdapter()

        await assertStopLeadsToDisconnected(actor)

        XCTAssertTrue(factory.adaptersCreated[1].isStopped, "The adapter of the current attempt must be stopped")
    }

    func testStopFromErrorState() async throws {
        let factory = GotaTunAdapterFactoryStub(outcome: .error(.internalError("test")))
        let actor = makeActor(adapterFactory: factory)

        await waitFor(actor, until: { $0.blockedReason != nil }, while: { await actor.start(options: launchOptions) })

        await assertStopLeadsToDisconnected(actor)
    }

    func testStopFromDisconnectedIsIgnored() async throws {
        let factory = GotaTunAdapterFactoryStub()
        let actor = makeActor(adapterFactory: factory)

        await waitFor(actor, until: { $0.isConnected }, while: { await actor.start(options: launchOptions) })

        await assertStopLeadsToDisconnected(actor)
        await assertStopLeadsToDisconnected(actor)

        XCTAssertEqual(factory.adaptersCreated.count, 1)
    }
}
