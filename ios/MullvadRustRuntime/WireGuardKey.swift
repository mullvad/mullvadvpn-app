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
import MullvadTypes

private let keyLength = 32

extension WireGuard.PrivateKey {
    /// Generate a new random private key via Rust FFI.
    public init() {
        var keyData = Data(repeating: 0, count: keyLength)
        keyData.withUnsafeMutableBytes { buffer in
            mullvad_generate_private_key(buffer.baseAddress!.assumingMemoryBound(to: UInt8.self))
        }
        self.init(rawValue: keyData)!
    }

    /// Derive the corresponding public key via Rust FFI.
    public var publicKey: WireGuard.PublicKey {
        rawValue.withUnsafeBytes { privateBuffer in
            var publicKeyData = Data(repeating: 0, count: keyLength)
            let privateKeyBytes = privateBuffer.baseAddress!.assumingMemoryBound(to: UInt8.self)
            publicKeyData.withUnsafeMutableBytes { publicBuffer in
                mullvad_derive_public_key(
                    privateKeyBytes,
                    publicBuffer.baseAddress!.assumingMemoryBound(to: UInt8.self)
                )
            }
            return WireGuard.PublicKey(rawValue: publicKeyData)!
        }
    }
}
