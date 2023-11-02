//
//  ShadowsocksCipher.swift
//  MullvadVPN
//
//  Created by pronebird on 13/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Type representing a shadowsocks cipher.
struct ShadowsocksCipher: RawRepresentable, CustomStringConvertible, Equatable, Hashable, Codable {
    let rawValue: String

    var description: String {
        rawValue
    }

    /// Default cipher.
    static let `default` = ShadowsocksCipher(rawValue: "chacha20")

    /// All supported ciphers.
    static let supportedCiphers = supportedCipherIdentifiers.map { ShadowsocksCipher(rawValue: $0) }
}

private let supportedCipherIdentifiers = [
    // Stream ciphers.
    "aes-128-cfb",
    "aes-128-cfb1",
    "aes-128-cfb8",
    "aes-128-cfb128",
    "aes-256-cfb",
    "aes-256-cfb1",
    "aes-256-cfb8",
    "aes-256-cfb128",
    "rc4",
    "rc4-md5",
    "chacha20",
    "salsa20",
    "chacha20-ietf",
    // AEAD ciphers.
    "aes-128-gcm",
    "aes-256-gcm",
    "chacha20-ietf-poly1305",
    "xchacha20-ietf-poly1305",
    "aes-128-pmac-siv",
    "aes-256-pmac-siv",
]
