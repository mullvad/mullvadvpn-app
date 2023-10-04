//
//  ActorTests.swift
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

final class ActorTests: XCTestCase {
    private var actor: PacketTunnelActor?
    private var stateSink: Combine.Cancellable?

    override func tearDown() async throws {
        stateSink?.cancel()
        actor?.stop()
        await actor?.waitUntilDisconnected()
    }

    /**
     Test a happy path start sequence.

     As actor should transition through the following states: .initial → .connecting → .connected
     */
    func testStart() async throws {
        let actor = PacketTunnelActor.mock()
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

        self.actor = actor

        actor.start(options: StartOptions(launchSource: .app))

        await fulfillment(of: allExpectations, timeout: 1, enforceOrder: true)
    }

    /**
     Test stopping connected tunnel.

     As actor should transition through the following states: .connected → .disconnecting → .disconnected
     */
    func testStopConnectedTunnel() async throws {
        let actor = PacketTunnelActor.mock()
        let connectedStateExpectation = expectation(description: "Expect connected state")
        let disconnectingStateExpectation = expectation(description: "Expect disconnecting state")
        let disconnectedStateExpectation = expectation(description: "Expect disconnected state")

        let allExpectations = [connectedStateExpectation, disconnectingStateExpectation, disconnectedStateExpectation]

        stateSink = await actor.$state
            .receive(on: DispatchQueue.main)
            .sink { newState in
                switch newState {
                case .connected:
                    connectedStateExpectation.fulfill()
                    actor.stop()

                case .disconnecting:
                    disconnectingStateExpectation.fulfill()

                case .disconnected:
                    disconnectedStateExpectation.fulfill()

                default:
                    break
                }
            }

        self.actor = actor

        actor.start(options: StartOptions(launchSource: .app))

        await fulfillment(of: allExpectations, timeout: 1, enforceOrder: true)
    }
}
