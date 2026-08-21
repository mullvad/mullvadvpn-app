// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadRustRuntime
import MullvadSettings
import MullvadTypes
import PacketTunnelCore

final public class EphemeralPeerExchangingPipeline {
    let keyExchanger: EphemeralPeerExchangeActorProtocol
    let onUpdateConfiguration: (EphemeralPeerNegotiationState) async -> Void
    let onFinish: () -> Void

    private var ephemeralPeerExchanger: EphemeralPeerExchangingProtocol!

    public init(
        _ keyExchanger: EphemeralPeerExchangeActorProtocol,
        onUpdateConfiguration: @escaping (EphemeralPeerNegotiationState) async -> Void,
        onFinish: @escaping () -> Void
    ) {
        self.keyExchanger = keyExchanger
        self.onUpdateConfiguration = onUpdateConfiguration
        self.onFinish = onFinish
    }

    public func startNegotiation(_ connectionState: ObservedConnectionState, privateKey: WireGuard.PrivateKey) async {
        keyExchanger.reset()
        let entryPeer = connectionState.selectedRelays.entry
        let exitPeer = connectionState.selectedRelays.exit
        let enablePostQuantum = connectionState.isPostQuantum
        let enableDaita = connectionState.isDaitaEnabled
        if let entryPeer {
            ephemeralPeerExchanger = MultiHopEphemeralPeerExchanger(
                entry: entryPeer,
                exit: exitPeer,
                devicePrivateKey: privateKey,
                keyExchanger: keyExchanger,
                enablePostQuantum: enablePostQuantum,
                enableDaita: enableDaita,
                onUpdateConfiguration: self.onUpdateConfiguration,
                onFinish: onFinish
            )
        } else {
            ephemeralPeerExchanger = SingleHopEphemeralPeerExchanger(
                exit: exitPeer,
                devicePrivateKey: privateKey,
                keyExchanger: keyExchanger,
                enablePostQuantum: enablePostQuantum,
                enableDaita: enableDaita,
                onUpdateConfiguration: self.onUpdateConfiguration,
                onFinish: onFinish
            )
        }
        await ephemeralPeerExchanger.start()
    }

    public func receivePostQuantumKey(
        _ key: WireGuard.PreSharedKey,
        ephemeralKey: WireGuard.PrivateKey,
        daitaParameters: DaitaV2Parameters?
    ) async {
        await ephemeralPeerExchanger.receivePostQuantumKey(
            key,
            ephemeralKey: ephemeralKey,
            daitaParameters: daitaParameters
        )
    }

    public func receiveEphemeralPeerPrivateKey(
        _ ephemeralPeerPrivateKey: WireGuard.PrivateKey,
        daitaParameters: DaitaV2Parameters?
    ) async {
        await ephemeralPeerExchanger.receiveEphemeralPeerPrivateKey(
            ephemeralPeerPrivateKey,
            daitaParameters: daitaParameters
        )
    }
}
