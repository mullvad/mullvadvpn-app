//
//  WireGuardKey.swift
//  MullvadRustRuntime
//
//  Created by Emils on 2026-04-07.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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
