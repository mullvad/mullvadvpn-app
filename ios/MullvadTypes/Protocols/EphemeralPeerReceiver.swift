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
import NetworkExtension

public class EphemeralPeerReceiver: EphemeralPeerReceiving, TunnelProvider {
    public func tunnelHandle() throws -> Int32 {
        try tunnelProvider.tunnelHandle()
    }

    public func wgFunctions() -> WgFunctionPointers {
        tunnelProvider.wgFunctions()
    }

    unowned let tunnelProvider: any TunnelProvider
    let keyReceiver: any EphemeralPeerReceiving

    public init(tunnelProvider: TunnelProvider, keyReceiver: any EphemeralPeerReceiving) {
        self.tunnelProvider = tunnelProvider
        self.keyReceiver = keyReceiver
    }

    // MARK: - EphemeralPeerReceiving

    public func receivePostQuantumKey(
        _ key: WireGuard.PreSharedKey,
        ephemeralKey: WireGuard.PrivateKey,
        daitaParameters: DaitaV2Parameters?
    ) {
        let semaphore = DispatchSemaphore(value: 0)
        Task {
            await keyReceiver.receivePostQuantumKey(key, ephemeralKey: ephemeralKey, daitaParameters: daitaParameters)
            semaphore.signal()
        }
        semaphore.wait()
    }

    public func receiveEphemeralPeerPrivateKey(
        _ ephemeralPeerPrivateKey: WireGuard.PrivateKey,
        daitaParameters: DaitaV2Parameters?
    ) {
        let semaphore = DispatchSemaphore(value: 0)
        Task {
            await keyReceiver.receiveEphemeralPeerPrivateKey(ephemeralPeerPrivateKey, daitaParameters: daitaParameters)
            semaphore.signal()
        }
        semaphore.wait()
    }

    public func ephemeralPeerExchangeFailed() {
        keyReceiver.ephemeralPeerExchangeFailed()
    }
}
