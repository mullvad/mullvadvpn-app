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

struct SingleHopPostQuantumKeyExchanging: PostQuantumKeyExchangingProtocol {
    let exit: SelectedRelay
    let keyExchanger: PostQuantumKeyExchangeActorProtocol
    let devicePrivateKey: PrivateKey
    let onFinish: () -> Void
    let onUpdateConfiguration: (PostQuantumNegotiationState) -> Void

    init(
        exit: SelectedRelay,
        devicePrivateKey: PrivateKey,
        keyExchanger: PostQuantumKeyExchangeActorProtocol,
        onUpdateConfiguration: @escaping (PostQuantumNegotiationState) -> Void,
        onFinish: @escaping () -> Void
    ) {
        self.devicePrivateKey = devicePrivateKey
        self.exit = exit
        self.keyExchanger = keyExchanger
        self.onUpdateConfiguration = onUpdateConfiguration
        self.onFinish = onFinish
    }

    func start() {
        onUpdateConfiguration(.single(PostQuantumConfigurationRelay(
            relay: exit,
            configuration: PostQuantumConfiguration(
                privateKey: devicePrivateKey,
                allowedIPs: [IPAddressRange(from: "\(LocalNetworkIPs.gatewayAddress.rawValue)/32")!]
            )
        )))
        keyExchanger.startNegotiation(with: devicePrivateKey)
    }

    func receivePostQuantumKey(
        _ preSharedKey: WireGuardKitTypes.PreSharedKey,
        ephemeralKey: WireGuardKitTypes.PrivateKey
    ) {
        onUpdateConfiguration(.single(PostQuantumConfigurationRelay(
            relay: exit,
            configuration: PostQuantumConfiguration(
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
