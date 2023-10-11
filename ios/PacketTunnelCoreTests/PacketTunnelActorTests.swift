//
//  PacketTunnelActorTests.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 05/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadSettings
import MullvadTypes
import Network
@testable import PacketTunnelCore
@testable import RelaySelector
import struct WireGuardKitTypes.IPAddressRange
import class WireGuardKitTypes.PrivateKey
import XCTest

final class PacketTunnelActorTests: XCTestCase {
    private var stateSink: Combine.Cancellable?

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

        stateSink = await actor.$state
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

        actor.start(options: StartOptions(launchSource: .app))

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

        stateSink = await actor.$state
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

        actor.start(options: StartOptions(launchSource: .app))
        actor.start(options: StartOptions(launchSource: .app))

        await fulfillment(of: allExpectations, timeout: 1, enforceOrder: true)
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
                    dnsServers: .gateway
                )
            }
        }

        let actor = PacketTunnelActor.mock(blockedStateErrorMapper: blockedStateMapper, settingsReader: settingsReader)

        stateSink = await actor.$state
            .receive(on: DispatchQueue.main)
            .sink { newState in
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

        actor.start(options: StartOptions(launchSource: .app))

        await fulfillment(of: allExpectations, timeout: 1, enforceOrder: true)
    }

    func testStopGoesToDisconnected() async throws {
        let actor = PacketTunnelActor.mock()
        let disconnectedStateExpectation = expectation(description: "Expect disconnected state")
        let connectedStateExpectation = expectation(description: "Expect connected state")

        let expression: (State) -> Bool = { if case .connected = $0 { true } else { false } }

        await expect(expression, on: actor) {
            connectedStateExpectation.fulfill()
        }

        // Wait for the connected state to happen so it doesn't get coalesced immediately after the call to `actor.stop`
        actor.start(options: StartOptions(launchSource: .app))
        await fulfillment(of: [connectedStateExpectation], timeout: 1)

        await expect(.disconnected, on: actor) {
            disconnectedStateExpectation.fulfill()
        }
        actor.stop()
        await fulfillment(of: [disconnectedStateExpectation], timeout: 1)
    }

    func testStopIsNoopBeforeStart() async throws {
        let actor = PacketTunnelActor.mock()

        actor.stop()
        actor.stop()
        actor.stop()

        switch await actor.state {
        case .initial: break
        default: XCTFail("Actor did not start, should be in .initial state")
        }
    }

    func testStopCancelsDefaultPathObserver() async throws {
        let pathObserver = DefaultPathObserverFake()
        let actor = PacketTunnelActor.mock(defaultPathObserver: pathObserver)

        let connectedStateExpectation = expectation(description: "Connected state")
        let didStopObserverExpectation = expectation(description: "Did stop path observer")
        didStopObserverExpectation.expectedFulfillmentCount = 2
        pathObserver.onStop = { didStopObserverExpectation.fulfill() }

        let expression: (State) -> Bool = { if case .connected = $0 { true } else { false } }

        await expect(expression, on: actor) {
            connectedStateExpectation.fulfill()
        }

        actor.start(options: StartOptions(launchSource: .app))
        await fulfillment(of: [connectedStateExpectation], timeout: 1)

        let disconnectedStateExpectation = expectation(description: "Disconnected state")

        await expect(.disconnected, on: actor) {
            disconnectedStateExpectation.fulfill()
        }
        actor.stop()
        await fulfillment(of: [disconnectedStateExpectation, didStopObserverExpectation], timeout: 1)
    }

    func testSetErrorStateGetsCancelled() async throws {
        let actor = PacketTunnelActor.mock()
        let connectingStateExpectation = expectation(description: "Connecting state")
        let disconnectedStateExpectation = expectation(description: "Disconnected state")

        stateSink = await actor.$state
            .receive(on: DispatchQueue.main)
            .sink { newState in
                switch newState {
                case .connecting:
                    // Guarantee that the task doesn't set the actor to error state before it's cancelled
                    let task = Task.detached {
                        try await Task.sleep(duration: .seconds(1))
                        actor.setErrorState(reason: .readSettings)
                    }
                    task.cancel()
                    connectingStateExpectation.fulfill()
                case .error:
                    XCTFail("Should not go to error state")
                case .disconnected:
                    disconnectedStateExpectation.fulfill()
                default:
                    break
                }
            }

        actor.start(options: StartOptions(launchSource: .app))
        await fulfillment(of: [connectingStateExpectation], timeout: 1)
        actor.stop()
        await fulfillment(of: [disconnectedStateExpectation], timeout: 1)
    }

    func testReconnectIsNoopBeforeConnecting() async throws {
        let actor = PacketTunnelActor.mock()
        let initialStateExpectation = expectation(description: "Expect initial state")

        stateSink = await actor.$state.receive(on: DispatchQueue.main).sink { newState in
            if case .initial = newState {
                initialStateExpectation.fulfill()
                return
            }
            XCTFail("Should not change states before starting the actor")
        }

        actor.reconnect(to: .random)

        await fulfillment(of: [initialStateExpectation], timeout: 1)
    }

    func testCannotReconnectAfterStopping() async throws {
        let actor = PacketTunnelActor.mock()

        let disconnectedStateExpectation = expectation(description: "Expect disconnected state")

        await expect(.disconnected, on: actor) {
            disconnectedStateExpectation.fulfill()
        }

        actor.start(options: StartOptions(launchSource: .app))
        actor.stop()

        await fulfillment(of: [disconnectedStateExpectation], timeout: 1)

        await expect(.initial, on: actor) {
            XCTFail("Should not be trying to reconnect after stopping")
        }
        actor.reconnect(to: .random)
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

        let expression: (State) -> Bool = { if case .connected = $0 { return true } else { return false } }
        await expect(expression, on: actor) {
            connectedExpectation.fulfill()
        }
        actor.start(options: StartOptions(launchSource: .app))
        await fulfillment(of: [connectedExpectation], timeout: 1)

        // Cancel the state sink to avoid overfulfilling the connected expectation
        stateSink?.cancel()

        actor.reconnect(to: .random)
        await fulfillment(of: [stopMonitorExpectation], timeout: 1)
    }
}

extension PacketTunnelActorTests {
    func expect(_ state: State, on actor: PacketTunnelActor, _ action: @escaping () -> Void) async {
        stateSink = await actor.$state.receive(on: DispatchQueue.main).sink { newState in
            if state == newState {
                action()
            }
        }
    }

    func expect(
        _ expression: @escaping (State) -> Bool,
        on actor: PacketTunnelActor,
        _ action: @escaping () -> Void
    ) async {
        stateSink = await actor.$state.receive(on: DispatchQueue.main).sink { newState in
            if expression(newState) {
                action()
            }
        }
    }
}
