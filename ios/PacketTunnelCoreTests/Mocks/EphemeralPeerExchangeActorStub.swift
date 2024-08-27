//
//  EphemeralPeerExchangeActorStub.swift
//  MullvadPostQuantumTests
//
//  Created by Mojgan on 2024-07-18.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
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
        switch result {
        case let .success((preSharedKey, ephemeralKey)):
            if enablePostQuantum {
                delegate?.receivePostQuantumKey(preSharedKey, ephemeralKey: ephemeralKey)
            } else {
                delegate?.receiveEphemeralPeerPrivateKey(ephemeralKey)
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
