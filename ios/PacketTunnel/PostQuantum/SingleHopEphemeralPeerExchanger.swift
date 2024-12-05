//
//  SingleHopPostQuantumKeyExchanging.swift
//  PacketTunnel
//
//  Created by Mojgan on 2024-07-15.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadRustRuntime
import MullvadSettings
import MullvadTypes
import PacketTunnelCore
import WireGuardKitTypes

struct SingleHopEphemeralPeerExchanger: EphemeralPeerExchangingProtocol {
    let exit: SelectedRelay
    let keyExchanger: EphemeralPeerExchangeActorProtocol
    let devicePrivateKey: PrivateKey
    let onFinish: () -> Void
    let onUpdateConfiguration: (EphemeralPeerNegotiationState) async -> Void
    let enablePostQuantum: Bool
    let enableDaita: Bool

    init(
        exit: SelectedRelay,
        devicePrivateKey: PrivateKey,
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
        await onUpdateConfiguration(.single(EphemeralPeerRelayConfiguration(
            relay: exit,
            configuration: EphemeralPeerConfiguration(
                privateKey: devicePrivateKey,
                allowedIPs: [IPAddressRange(from: "\(LocalNetworkIPs.gatewayAddress.rawValue)/32")!]
            )
        )))
        keyExchanger.startNegotiation(
            with: devicePrivateKey,
            enablePostQuantum: enablePostQuantum,
            enableDaita: enableDaita
        )
    }

    public func receiveEphemeralPeerPrivateKey(_ ephemeralKey: PrivateKey) async {
        await onUpdateConfiguration(.single(EphemeralPeerRelayConfiguration(
            relay: exit,
            configuration: EphemeralPeerConfiguration(
                privateKey: ephemeralKey,
                preSharedKey: nil,
                allowedIPs: [
                    IPAddressRange(from: "\(LocalNetworkIPs.defaultRouteIpV4.rawValue)/0")!,
                    IPAddressRange(from: "\(LocalNetworkIPs.defaultRouteIpV6.rawValue)/0")!,
                ]
            )
        )))
        self.onFinish()
    }

    func receivePostQuantumKey(
        _ preSharedKey: WireGuardKitTypes.PreSharedKey,
        ephemeralKey: WireGuardKitTypes.PrivateKey
    ) async {
        await onUpdateConfiguration(.single(EphemeralPeerRelayConfiguration(
            relay: exit,
            configuration: EphemeralPeerConfiguration(
                privateKey: ephemeralKey,
                preSharedKey: preSharedKey,
                allowedIPs: [
                    IPAddressRange(from: "\(LocalNetworkIPs.defaultRouteIpV4.rawValue)/0")!,
                    IPAddressRange(from: "\(LocalNetworkIPs.defaultRouteIpV6.rawValue)/0")!,
                ]
            )
        )))
        self.onFinish()
    }
}
