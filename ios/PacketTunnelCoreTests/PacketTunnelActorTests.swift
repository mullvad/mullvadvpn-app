//
//  PacketTunnelActorTests.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 05/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
@testable import MullvadREST
@testable import MullvadSettings
import MullvadTypes
import Network
@testable import PacketTunnelCore
import WireGuardKitTypes
import XCTest

final class PacketTunnelActorTests: XCTestCase {
    private var stateSink: Combine.Cancellable?
    private let launchOptions = StartOptions(launchSource: .app)

    override func tearDown() async throws {
        stateSink?.cancel()
    }

    /**
     Test a happy path start sequence.

     As actor should transition through the following states: .initial → .connecting → .connected
     */
    func testStartGoesToConnectedInSequence() async throws {
        let actor = PacketTunnelActor.mock()

        // As actor starts it should transition through the following states based on simulation:
        // .initial → .connecting → .connected
        let initialStateExpectation = expectation(description: "Expect initial state")
        let connectingExpectation = expectation(description: "Expect connecting state")
        let connectedStateExpectation = expectation(description: "Expect connected state")

        let allExpectations = [initialStateExpectation, connectingExpectation, connectedStateExpectation]

        stateSink = await actor.$observedState
            .receive(on: DispatchQueue.main)
            .sink { newState in
                switch newState {
                case .initial:
                    initialStateExpectation.fulfill()
                case .connecting:
                    connectingExpectation.fulfill()
                case .connected:
                    connectedStateExpectation.fulfill()
                default:
                    break
                }
            }

        actor.start(options: launchOptions)

        await fulfillment(of: allExpectations, timeout: 1, enforceOrder: true)
    }

    func testStartIgnoresSubsequentStarts() async throws {
        let actor = PacketTunnelActor.mock()

        // As actor starts it should transition through the following states based on simulation:
        // .initial → .connecting → .connected
        let initialStateExpectation = expectation(description: "Expect initial state")
        let connectingExpectation = expectation(description: "Expect connecting state")
        let connectedStateExpectation = expectation(description: "Expect connected state")

        let allExpectations = [initialStateExpectation, connectingExpectation, connectedStateExpectation]

        stateSink = await actor.$observedState
            .receive(on: DispatchQueue.main)
            .sink { newState in
                switch newState {
                case .initial:
                    initialStateExpectation.fulfill()
                case .connecting:
                    connectingExpectation.fulfill()
                case .connected:
                    connectedStateExpectation.fulfill()
                default:
                    break
                }
            }

        actor.start(options: launchOptions)
        actor.start(options: launchOptions)

        await fulfillment(of: allExpectations, timeout: 1, enforceOrder: true)
    }

    /**
     Each subsequent connection attempt should produce a single change to `state` containing the incremented attempt counter and new relay.
     .connecting (attempt: 0) → .connecting (attempt: 1) → .connecting (attempt: 2) → ...
     */
    func testConnectionAttemptTransition() async throws {
        let tunnelMonitor = TunnelMonitorStub { _, _ in }
        let actor = PacketTunnelActor.mock(tunnelMonitor: tunnelMonitor)
        let connectingStateExpectation = expectation(description: "Expect connecting state")
        connectingStateExpectation.expectedFulfillmentCount = 5
        var nextAttemptCount: UInt = 0
        stateSink = await actor.$observedState
            .receive(on: DispatchQueue.main)
            .sink { newState in
                switch newState {
                case .initial:
                    break
                case let .connecting(connState):
                    XCTAssertEqual(connState.connectionAttemptCount, nextAttemptCount)
                    nextAttemptCount += 1
                    connectingStateExpectation.fulfill()
                    if nextAttemptCount < connectingStateExpectation.expectedFulfillmentCount {
                        tunnelMonitor.dispatch(.connectionLost, after: .milliseconds(10))
                    }
                default:
                    XCTFail("Received invalid state: \(newState.name).")
                }
            }

        actor.start(options: StartOptions(launchSource: .app))
        await fulfillment(of: [connectingStateExpectation], timeout: 1)
    }

    /**
     Each subsequent re-connection attempt should produce a single change to `state` containing the incremented attempt counter and new relay.
     .reconnecting (attempt: 0) → .reconnecting (attempt: 1) → .reconnecting (attempt: 2) → ...
     */
    func testReconnectionAttemptTransition() async throws {
        let tunnelMonitor = TunnelMonitorStub { _, _ in }
        let actor = PacketTunnelActor.mock(tunnelMonitor: tunnelMonitor)
        let connectingStateExpectation = expectation(description: "Expect connecting state")
        let connectedStateExpectation = expectation(description: "Expect connected state")
        let reconnectingStateExpectation = expectation(description: "Expect reconnecting state")
        reconnectingStateExpectation.expectedFulfillmentCount = 5
        var nextAttemptCount: UInt = 0
        stateSink = await actor.$observedState
            .receive(on: DispatchQueue.main)
            .sink { newState in
                switch newState {
                case .initial:
                    break
                case .connecting:
                    connectingStateExpectation.fulfill()
                    tunnelMonitor.dispatch(.connectionEstablished, after: .milliseconds(10))
                case .connected:
                    connectedStateExpectation.fulfill()
                    tunnelMonitor.dispatch(.connectionLost, after: .milliseconds(10))
                case let .reconnecting(connState):
                    XCTAssertEqual(connState.connectionAttemptCount, nextAttemptCount)
                    nextAttemptCount += 1
                    reconnectingStateExpectation.fulfill()
                    if nextAttemptCount < reconnectingStateExpectation.expectedFulfillmentCount {
                        tunnelMonitor.dispatch(.connectionLost, after: .milliseconds(10))
                    }
                default:
                    XCTFail("Received invalid state: \(newState.name).")
                }
            }

        actor.start(options: StartOptions(launchSource: .app))
        await fulfillment(
            of: [connectingStateExpectation, connectedStateExpectation, reconnectingStateExpectation],
            timeout: 1,
            enforceOrder: true
        )
    }

    /**
     Test start sequence when reading settings yields an error indicating that device is locked.
     This is common when network extenesion starts on boot with iOS.

     1. The first attempt to read settings yields an error indicating that device is locked.
     2. An actor should set up a task to reconnect the tunnel periodically.
     3. The issue goes away on the second attempt to read settings.
     4. An actor should transition through `.connecting` towards`.connected` state.
     */
    func testLockedDeviceErrorOnBoot() async throws {
        let initialStateExpectation = expectation(description: "Expect initial state")
        let errorStateExpectation = expectation(description: "Expect error state")
        let connectingStateExpectation = expectation(description: "Expect connecting state")
        let connectedStateExpectation = expectation(description: "Expect connected state")
        let allExpectations = [
            initialStateExpectation,
            errorStateExpectation,
            connectingStateExpectation,
            connectedStateExpectation,
        ]

        let blockedStateMapper = BlockedStateErrorMapperStub { error in
            if let error = error as? POSIXError, error.code == .EPERM {
                return .deviceLocked
            } else {
                return .unknown
            }
        }

        var isFirstReadAttempt = true
        let settingsReader = SettingsReaderStub {
            if isFirstReadAttempt {
                isFirstReadAttempt = false
                throw POSIXError(.EPERM)
            } else {
                return Settings(
                    privateKey: PrivateKey(),
                    interfaceAddresses: [IPAddressRange(from: "127.0.0.1/32")!],
                    relayConstraints: RelayConstraints(),
                    dnsServers: .gateway,
                    obfuscation: WireGuardObfuscationSettings(state: .off, port: .automatic)
                )
            }
        }

        let actor = PacketTunnelActor.mock(blockedStateErrorMapper: blockedStateMapper, settingsReader: settingsReader)

        stateSink = await actor.$observedState.receive(on: DispatchQueue.main).sink { newState in
            switch newState {
            case .initial:
                initialStateExpectation.fulfill()
            case .error:
                errorStateExpectation.fulfill()
            case .connecting:
                connectingStateExpectation.fulfill()
            case .connected:
                connectedStateExpectation.fulfill()
            default:
                break
            }
        }

        actor.start(options: launchOptions)

        await fulfillment(of: allExpectations, timeout: 1, enforceOrder: true)
    }

    func testStopGoesToDisconnected() async throws {
        let actor = PacketTunnelActor.mock()
        let disconnectedStateExpectation = expectation(description: "Expect disconnected state")
        let connectedStateExpectation = expectation(description: "Expect connected state")

        let expression: (ObservedState) -> Bool = { if case .connected = $0 { true } else { false } }

        await expect(expression, on: actor) {
            connectedStateExpectation.fulfill()
        }

        // Wait for the connected state to happen so it doesn't get coalesced immediately after the call to `actor.stop`
        actor.start(options: launchOptions)
        await fulfillment(of: [connectedStateExpectation], timeout: 1)

        await expect(.disconnected, on: actor) {
            disconnectedStateExpectation.fulfill()
        }
        actor.stop()
        await fulfillment(of: [disconnectedStateExpectation], timeout: 1)
    }

    func testStopIsNoopBeforeStart() async throws {
        let actor = PacketTunnelActor.mock()

        let disconnectedExpectation = expectation(description: "Disconnected state")
        disconnectedExpectation.isInverted = true

        await expect(.disconnected, on: actor) {
            disconnectedExpectation.fulfill()
        }

        actor.stop()
        actor.stop()
        actor.stop()

        await fulfillment(of: [disconnectedExpectation], timeout: Duration.milliseconds(100).timeInterval)
    }

    func testStopCancelsDefaultPathObserver() async throws {
        let pathObserver = DefaultPathObserverFake()
        let actor = PacketTunnelActor.mock(defaultPathObserver: pathObserver)

        let connectedStateExpectation = expectation(description: "Connected state")
        let didStopObserverExpectation = expectation(description: "Did stop path observer")
        didStopObserverExpectation.expectedFulfillmentCount = 2
        pathObserver.onStop = { didStopObserverExpectation.fulfill() }

        let expression: (ObservedState) -> Bool = { if case .connected = $0 { true } else { false } }

        await expect(expression, on: actor) {
            connectedStateExpectation.fulfill()
        }

        actor.start(options: launchOptions)
        await fulfillment(of: [connectedStateExpectation], timeout: 1)

        let disconnectedStateExpectation = expectation(description: "Disconnected state")

        await expect(.disconnected, on: actor) {
            disconnectedStateExpectation.fulfill()
        }
        actor.stop()
        await fulfillment(of: [disconnectedStateExpectation, didStopObserverExpectation], timeout: 1)
    }

    func testCannotEnterErrorStateWhenStopping() async throws {
        let actor = PacketTunnelActor.mock()
        let connectingStateExpectation = expectation(description: "Connecting state")
        let disconnectedStateExpectation = expectation(description: "Disconnected state")
        let errorStateExpectation = expectation(description: "Should not enter error state")
        errorStateExpectation.isInverted = true

        /// Because of how commands are processed by the actor's `CommandChannel`
        /// `start` and `stop` cannot be chained together, otherwise there is a risk that the `start` command
        /// gets coalesced by the `stop` command, and leaves the actor in its `.initial` state.
        /// Guarantee here that the actor reaches the `.connecting` state before moving on.
        let expression: (ObservedState) -> Bool = { if case .connecting = $0 { true } else { false } }
        await expect(expression, on: actor) {
            connectingStateExpectation.fulfill()
        }
        actor.start(options: launchOptions)
        await fulfillment(of: [connectingStateExpectation], timeout: 1)

        stateSink = await actor.$observedState
            .receive(on: DispatchQueue.main)
            .sink { newState in
                switch newState {
                case .error:
                    errorStateExpectation.fulfill()
                case .disconnected:
                    disconnectedStateExpectation.fulfill()
                default:
                    break
                }
            }

        actor.stop()
        actor.setErrorState(reason: .readSettings)

        await fulfillment(of: [disconnectedStateExpectation], timeout: 1)
        await fulfillment(of: [errorStateExpectation], timeout: Duration.milliseconds(100).timeInterval)
    }

    func testReconnectIsNoopBeforeConnecting() async throws {
        let actor = PacketTunnelActor.mock()
        let reconnectingStateExpectation = expectation(description: "Expect initial state")
        reconnectingStateExpectation.isInverted = true

        let expression: (ObservedState) -> Bool = { if case .reconnecting = $0 { true } else { false } }

        await expect(expression, on: actor) {
            reconnectingStateExpectation.fulfill()
        }

        actor.reconnect(to: .random)

        await fulfillment(
            of: [reconnectingStateExpectation],
            timeout: Duration.milliseconds(100).timeInterval
        )
    }

    func testCannotReconnectAfterStopping() async throws {
        let actor = PacketTunnelActor.mock()

        let connectedStateExpectation = expectation(description: "Expect connected state")
        let connectedState: (ObservedState) -> Bool = { if case .connected = $0 { true } else { false } }
        await expect(connectedState, on: actor) {
            connectedStateExpectation.fulfill()
        }

        actor.start(options: launchOptions)
        // Wait for the connected state to happen so it doesn't get coalesced immediately after the call to `actor.stop`
        await fulfillment(of: [connectedStateExpectation], timeout: 1)

        let disconnectedStateExpectation = expectation(description: "Expect disconnected state")
        await expect(.disconnected, on: actor) { disconnectedStateExpectation.fulfill() }
        actor.stop()
        await fulfillment(of: [disconnectedStateExpectation], timeout: 1)

        let reconnectingStateExpectation = expectation(description: "Expect reconnecting state")
        reconnectingStateExpectation.isInverted = true
        let reconnectingState: (ObservedState) -> Bool = { if case .reconnecting = $0 { true } else { false } }
        await expect(reconnectingState, on: actor) { reconnectingStateExpectation.fulfill() }

        actor.reconnect(to: .random)
        await fulfillment(
            of: [reconnectingStateExpectation],
            timeout: Duration.milliseconds(100).timeInterval
        )
    }

    func testReconnectionStopsTunnelMonitor() async throws {
        let stopMonitorExpectation = expectation(description: "Tunnel monitor stop")

        let tunnelMonitor = TunnelMonitorStub { command, dispatcher in
            switch command {
            case .start:
                dispatcher.send(.connectionEstablished, after: .milliseconds(10))
            case .stop:
                stopMonitorExpectation.fulfill()
            }
        }
        let actor = PacketTunnelActor.mock(tunnelMonitor: tunnelMonitor)
        let connectedExpectation = expectation(description: "Expect connected state")

        let expression: (ObservedState) -> Bool = { if case .connected = $0 { return true } else { return false } }
        await expect(expression, on: actor) {
            connectedExpectation.fulfill()
        }
        actor.start(options: launchOptions)
        await fulfillment(of: [connectedExpectation], timeout: 1)

        // Cancel the state sink to avoid overfulfilling the connected expectation
        stateSink?.cancel()

        actor.reconnect(to: .random)
        await fulfillment(of: [stopMonitorExpectation], timeout: 1)
    }
}

extension PacketTunnelActorTests {
    func expect(_ state: ObservedState, on actor: PacketTunnelActor, _ action: @escaping () -> Void) async {
        stateSink = await actor.$observedState.receive(on: DispatchQueue.main).sink { newState in
            if state == newState {
                action()
            }
        }
    }

    func expect(
        _ expression: @escaping (ObservedState) -> Bool,
        on actor: PacketTunnelActor,
        _ action: @escaping () -> Void
    ) async {
        stateSink = await actor.$observedState.receive(on: DispatchQueue.main).sink { newState in
            if expression(newState) {
                action()
            }
        }
    }
}

// swiftlint:disable:this file_length
