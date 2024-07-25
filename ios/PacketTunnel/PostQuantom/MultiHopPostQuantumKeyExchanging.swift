//
//  MultiHopPostQuantumKeyExchanging.swift
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

final class MultiHopPostQuantumKeyExchanging: PostQuantumKeyExchangingProtocol {
    let entry: SelectedRelay
    let exit: SelectedRelay
    let keyExchanger: PostQuantumKeyExchangeActorProtocol
    let devicePrivateKey: PrivateKey
    let onFinish: () -> Void
    let onUpdateConfiguration: (PostQuantumNegotiationState) -> Void

    private var entryPostQuantumKey: PostQuantumKey!
    private var exitPostQuantumKey: PostQuantumKey!

    private let defaultGatewayAddressRange = [IPAddressRange(from: "10.64.0.1/32")!]
    private let allTrafficRange = [IPAddressRange(from: "0.0.0.0/0")!, IPAddressRange(from: "::/0")!]

    private var state: StateMachine = .initial

    enum StateMachine {
        case initial
        case negotiatingWithEntry
        case negotiatingBetweenEntryAndExit
        case makeConnection
        case connected
    }

    init(
        entry: SelectedRelay,
        exit: SelectedRelay,
        devicePrivateKey: PrivateKey,
        keyExchanger: PostQuantumKeyExchangeActorProtocol,
        onUpdateConfiguration: @escaping (PostQuantumNegotiationState) -> Void,
        onFinish: @escaping () -> Void
    ) {
        self.entry = entry
        self.exit = exit
        self.devicePrivateKey = devicePrivateKey
        self.keyExchanger = keyExchanger
        self.onUpdateConfiguration = onUpdateConfiguration
        self.onFinish = onFinish
    }

    func start() {
        guard state == .initial else { return }
        negotiateWithEntry()
    }

    func receivePostQuantumKey(
        _ preSharedKey: PreSharedKey,
        ephemeralKey: PrivateKey
    ) {
        if state == .negotiatingWithEntry {
            entryPostQuantumKey = PostQuantumKey(preSharedKey: preSharedKey, ephemeralKey: ephemeralKey)
            negotiateBetweenEntryAndExit()
        } else if state == .negotiatingBetweenEntryAndExit {
            exitPostQuantumKey = PostQuantumKey(preSharedKey: preSharedKey, ephemeralKey: ephemeralKey)
            makeConnection()
        }
    }

    private func negotiateWithEntry() {
        state = .negotiatingWithEntry
        onUpdateConfiguration(.single(PostQuantumConfigurationRelay(
            relay: entry,
            configuration: PostQuantumConfiguration(
                privateKey: devicePrivateKey,
                allowedIPs: defaultGatewayAddressRange
            )
        )))
        keyExchanger.startNegotiation(with: devicePrivateKey)
    }

    private func negotiateBetweenEntryAndExit() {
        state = .negotiatingBetweenEntryAndExit
        onUpdateConfiguration(.multi(
            entry: PostQuantumConfigurationRelay(
                relay: entry,
                configuration: PostQuantumConfiguration(
                    privateKey: entryPostQuantumKey.ephemeralKey,
                    preSharedKey: entryPostQuantumKey.preSharedKey,
                    allowedIPs: [IPAddressRange(from: "\(exit.endpoint.ipv4Relay.ip)/32")!]
                )
            ),
            exit: PostQuantumConfigurationRelay(
                relay: exit,
                configuration: PostQuantumConfiguration(
                    privateKey: devicePrivateKey,
                    allowedIPs: defaultGatewayAddressRange
                )
            )
        ))
        keyExchanger.startNegotiation(with: devicePrivateKey)
    }

    private func makeConnection() {
        state = .makeConnection
        onUpdateConfiguration(.multi(
            entry: PostQuantumConfigurationRelay(
                relay: entry,
                configuration: PostQuantumConfiguration(
                    privateKey: entryPostQuantumKey.ephemeralKey,
                    preSharedKey: entryPostQuantumKey.preSharedKey,
                    allowedIPs: [IPAddressRange(from: "\(exit.endpoint.ipv4Relay.ip)/32")!]
                )
            ),
            exit: PostQuantumConfigurationRelay(
                relay: exit,
                configuration: PostQuantumConfiguration(
                    privateKey: exitPostQuantumKey.ephemeralKey,
                    preSharedKey: exitPostQuantumKey.preSharedKey,
                    allowedIPs: allTrafficRange
                )
            )
        ))
        self.onFinish()
    }
}
