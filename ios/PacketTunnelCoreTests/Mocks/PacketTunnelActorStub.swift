//
//  PacketTunnelActorStub.swift
//  PacketTunnelCoreTests
//
//  Created by Jon Petersson on 2023-10-11.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import Network
import PacketTunnelCore
import XCTest

struct PacketTunnelActorStub: PacketTunnelActorProtocol {
    var observedStates: AsyncStream<ObservedState> {
        get async {
            AsyncStream { continuation in
                continuation.yield(innerState)
                continuation.finish()
            }
        }
    }

    func start(options: StartOptions) {}
    func stop() {}
    func waitUntilDisconnected() async {}
    func onSleep() {}
    func onWake() {}
    func updateNetworkReachability(networkPathStatus: NWPath.Status) {}
    func setErrorState(reason: BlockedStateReason) {}
    func notifyEphemeralPeerNegotiated() {}

    func changeEphemeralPeerNegotiationState(
        configuration: EphemeralPeerNegotiationState,
        reconfigurationSemaphore: OneshotChannel
    ) {}

    let innerState: ObservedState = .disconnected
    var stateExpectation: XCTestExpectation?
    var reconnectExpectation: XCTestExpectation?
    var keyRotationExpectation: XCTestExpectation?

    var observedState: ObservedState {
        get async {
            stateExpectation?.fulfill()
            return innerState
        }
    }

    func reconnect(to nextRelays: NextRelays, reconnectReason: ActorReconnectReason) {
        reconnectExpectation?.fulfill()
    }

    func notifyKeyRotation(date: Date?) {
        keyRotationExpectation?.fulfill()
    }
}
