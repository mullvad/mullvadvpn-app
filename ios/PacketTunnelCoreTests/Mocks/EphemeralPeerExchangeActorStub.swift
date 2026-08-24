// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import NetworkExtension

@testable import MullvadRustRuntime
@testable import MullvadTypes
@testable import PacketTunnelCore

final class EphemeralPeerExchangeActorStub: EphemeralPeerExchangeActorProtocol, @unchecked Sendable {
    typealias KeyNegotiationResult = Result<
        (WireGuard.PreSharedKey, WireGuard.PrivateKey), EphemeralPeerExchangeErrorStub
    >
    var result: KeyNegotiationResult = .failure(.unknown)

    var delegate: EphemeralPeerReceiving?

    func startNegotiation(with privateKey: WireGuard.PrivateKey, enablePostQuantum: Bool, enableDaita: Bool) {
        let daita =
            enableDaita
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
                Task {
                    await delegate?.receivePostQuantumKey(
                        preSharedKey,
                        ephemeralKey: ephemeralKey,
                        daitaParameters: daita
                    )
                }
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
