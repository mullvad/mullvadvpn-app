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

public protocol PacketTunnelActorProtocol {
    // State observation
    var observedState: ObservedState { get async }
    var observedStates: AsyncStream<ObservedState> { get async }

    // Lifecycle
    func start(options: StartOptions) async
    func stop() async
    func waitUntilDisconnected() async

    // Sleep cycle
    func onSleep() async
    func onWake()

    // Network
    func updateNetworkReachability(networkPathStatus: NWPath.Status) async

    // Reconnection & key rotation
    func reconnect(to nextRelays: NextRelays, reconnectReason: ActorReconnectReason) async
    func notifyKeyRotation(date: Date?) async

    // Error state
    func setErrorState(reason: BlockedStateReason) async

    // Ephemeral peer negotiation
    func notifyEphemeralPeerNegotiated() async
    func changeEphemeralPeerNegotiationState(
        configuration: EphemeralPeerNegotiationState,
        reconfigurationSemaphore: OneshotChannel
    ) async
}
