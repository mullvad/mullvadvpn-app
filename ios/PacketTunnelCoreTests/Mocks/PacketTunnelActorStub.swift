// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
