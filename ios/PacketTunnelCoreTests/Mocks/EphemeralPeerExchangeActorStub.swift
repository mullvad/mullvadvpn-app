//
//  EphemeralPeerExchangeActorStub.swift
//  MullvadPostQuantumTests
//
//  Created by Mojgan on 2024-07-18.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadRustRuntime
@testable import MullvadTypes
import NetworkExtension
@testable import PacketTunnelCore
@testable import WireGuardKitTypes

final class EphemeralPeerExchangeActorStub: EphemeralPeerExchangeActorProtocol {
    typealias KeyNegotiationResult = Result<(PreSharedKey, PrivateKey), EphemeralPeerExchangeErrorStub>
    var result: KeyNegotiationResult = .failure(.unknown)

    var delegate: EphemeralPeerReceiving?

    func startNegotiation(with privateKey: PrivateKey, enablePostQuantum: Bool, enableDaita: Bool) {
        let daita = enableDaita
            ? DaitaV2Parameters(
                machines: "test",
                maximumEvents: 1,
                maximumActions: 1,
                maximumPadding: 1.0,
                maximumBlocking: 1.0
            )
            : nil
        switch result {
        case let .success((preSharedKey, ephemeralKey)):
            if enablePostQuantum {
                Task { await delegate?.receivePostQuantumKey(
                    preSharedKey,
                    ephemeralKey: ephemeralKey,
                    daitaParameters: daita
                ) }
            } else {
                Task { await delegate?.receiveEphemeralPeerPrivateKey(ephemeralKey, daitaParameters: daita) }
            }
        case .failure:
            delegate?.ephemeralPeerExchangeFailed()
        }
    }

    func endCurrentNegotiation() {}

    func reset() {}
}

enum EphemeralPeerExchangeErrorStub: Error {
    case unknown
    case canceled
}
