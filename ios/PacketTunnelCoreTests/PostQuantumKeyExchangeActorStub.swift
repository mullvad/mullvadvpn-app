//
//  PostQuantumKeyExchangeActorStub.swift
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

final class PostQuantumKeyExchangeActorStub: PostQuantumKeyExchangeActorProtocol {
    typealias KeyNegotiationResult = Result<(PreSharedKey, PrivateKey), PostQuantumKeyExchangeErrorStub>
    var result: KeyNegotiationResult = .failure(.unknown)

    var delegate: PostQuantumKeyReceiving?

    func startNegotiation(with privateKey: PrivateKey) {
        switch result {
        case let .success((preSharedKey, ephemeralKey)):
            delegate?.receivePostQuantumKey(preSharedKey, ephemeralKey: ephemeralKey)
        case .failure:
            delegate?.keyExchangeFailed()
        }
    }

    func endCurrentNegotiation() {}

    func reset() {}
}

enum PostQuantumKeyExchangeErrorStub: Error {
    case unknown
    case canceled
}
