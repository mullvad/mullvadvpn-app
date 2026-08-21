// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadREST
import MullvadRustRuntime
import MullvadSettings
import MullvadTypes
import PacketTunnelCore

struct SingleHopEphemeralPeerExchanger: EphemeralPeerExchangingProtocol {
    let exit: SelectedRelay
    let keyExchanger: EphemeralPeerExchangeActorProtocol
    let devicePrivateKey: WireGuard.PrivateKey
    let onFinish: () -> Void
    let onUpdateConfiguration: (EphemeralPeerNegotiationState) async -> Void
    let enablePostQuantum: Bool
    let enableDaita: Bool

    init(
        exit: SelectedRelay,
        devicePrivateKey: WireGuard.PrivateKey,
        keyExchanger: EphemeralPeerExchangeActorProtocol,
        enablePostQuantum: Bool,
        enableDaita: Bool,
        onUpdateConfiguration: @escaping (EphemeralPeerNegotiationState) async -> Void,
        onFinish: @escaping () -> Void
    ) {
        self.devicePrivateKey = devicePrivateKey
        self.exit = exit
        self.keyExchanger = keyExchanger
        self.enablePostQuantum = enablePostQuantum
        self.enableDaita = enableDaita
        self.onUpdateConfiguration = onUpdateConfiguration
        self.onFinish = onFinish
    }

    func start() async {
        await onUpdateConfiguration(
            .single(
                EphemeralPeerRelayConfiguration(
                    relay: exit,
                    configuration: EphemeralPeerConfiguration(
                        privateKey: devicePrivateKey,
                        allowedIPs: [IPAddressRange(from: "\(LocalNetworkIPs.gatewayAddressIpV4.rawValue)/32")!],
                        daitaParameters: nil
                    )
                )))
        keyExchanger.startNegotiation(
            with: devicePrivateKey,
            enablePostQuantum: enablePostQuantum,
            enableDaita: enableDaita
        )
    }

    public func receiveEphemeralPeerPrivateKey(
        _ ephemeralKey: WireGuard.PrivateKey, daitaParameters: DaitaV2Parameters?
    ) async {
        await onUpdateConfiguration(
            .single(
                EphemeralPeerRelayConfiguration(
                    relay: exit,
                    configuration: EphemeralPeerConfiguration(
                        privateKey: ephemeralKey,
                        preSharedKey: nil,
                        allowedIPs: [
                            IPAddressRange(from: "\(LocalNetworkIPs.defaultRouteIpV4.rawValue)/0")!,
                            IPAddressRange(from: "\(LocalNetworkIPs.defaultRouteIpV6.rawValue)/0")!,
                        ],
                        daitaParameters: daitaParameters
                    )
                )))
        self.onFinish()
    }

    func receivePostQuantumKey(
        _ preSharedKey: WireGuard.PreSharedKey,
        ephemeralKey: WireGuard.PrivateKey,
        daitaParameters: DaitaV2Parameters?
    ) async {
        await onUpdateConfiguration(
            .single(
                EphemeralPeerRelayConfiguration(
                    relay: exit,
                    configuration: EphemeralPeerConfiguration(
                        privateKey: ephemeralKey,
                        preSharedKey: preSharedKey,
                        allowedIPs: [
                            IPAddressRange(from: "\(LocalNetworkIPs.defaultRouteIpV4.rawValue)/0")!,
                            IPAddressRange(from: "\(LocalNetworkIPs.defaultRouteIpV6.rawValue)/0")!,
                        ],
                        daitaParameters: daitaParameters
                    )
                )))
        self.onFinish()
    }
}
