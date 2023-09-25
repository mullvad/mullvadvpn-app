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
        await actor?.stop()
    }

    /**
     Test a happy path start sequence.

     As actor should transition through the following states: .initial → .connecting → .connected
     */
    func testStart() async throws {
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

        self.actor = actor

        try await actor.start(options: StartOptions(launchSource: .app))

        await fulfillment(of: allExpectations, timeout: 1, enforceOrder: true)
    }

    /**
     Test start sequence when reading settings yields an error indicating that device is locked.
     This is common when network extenesion starts on boot with iOS.

     1. The frist attempt to read settings yields an error indicating that device is locked.
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

        let blockedStateMapper = MockBlockedStateErrorMapper { error in
            if let error = error as? POSIXError, error.code == .EPERM {
                return .deviceLocked
            } else {
                return .unknown
            }
        }

        var isFirstReadAttempt = true
        let settingsReader = MockSettingsReader {
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

        self.actor = actor

        try await actor.start(options: StartOptions(launchSource: .app))

        await fulfillment(of: allExpectations, timeout: 200, enforceOrder: true)
    }
}
