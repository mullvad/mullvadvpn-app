//
//  Curve25519.swift
//  MullvadVPN
//
//  Created by pronebird on 18/06/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//  Copyright © 2018-2019 WireGuard LLC. All Rights Reserved.
//

import Foundation

struct Curve25519 {

    static let keyLength: Int = 32

    static func generatePrivateKey() -> Data {
        var privateKey = [UInt8](repeating: 0, count: keyLength)
        privateKey.withUnsafeMutableBufferPointer { (ptr) in
            curve25519_generate_private_key(ptr.baseAddress!)
        }
        return Data(privateKey)
    }

    static func generatePublicKey(fromPrivateKey privateKey: Data) -> Data {
        assert(privateKey.count == Self.keyLength)

        var publicKey = [UInt8](repeating: 0, count: keyLength)
        privateKey.withUnsafeBytes { (privateKeyBytes) in
            let privateKeyBytesPointer = privateKeyBytes.bindMemory(to: UInt8.self)

            publicKey.withUnsafeMutableBufferPointer { (publicKeyPointer) in
                curve25519_derive_public_key(
                    publicKeyPointer.baseAddress!,
                    privateKeyBytesPointer.baseAddress!
                )
            }
        }

        return Data(publicKey)
    }
}
