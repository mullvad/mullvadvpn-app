//
//  ShadowsocksCipherOptions.swift
//  MullvadVPN
//
//  Created by pronebird on 13/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Type representing a shadowsocks cipher.
struct ShadowsocksCipherOptions: RawRepresentable, Codable, Hashable {
    let rawValue: CipherIdentifiers

    /// Default cipher.
    static let `default` = ShadowsocksCipherOptions(rawValue: .CHACHA20)

    /// All supported ciphers.
    static let all = CipherIdentifiers.allCases.map { ShadowsocksCipherOptions(rawValue: $0) }
}

enum CipherIdentifiers: String, CaseIterable, CustomStringConvertible, Codable {
    // Stream ciphers.
    case CFB_AES128 = "aes-128-cfb"
    case CFB1_AES128 = "aes-128-cfb1"
    case CFB8_AES128 = "aes-128-cfb8"
    case CFB128_AES128 = "aes-128-cfb128"
    case CFB_AES256 = "aes-256-cfb"
    case CFB1_AES256 = "aes-256-cfb1"
    case CFB8_AES256 = "aes-256-cfb8"
    case CFB128_AES256 = "aes-256-cfb128"
    case RC4 = "rc4"
    case RC4_MD5 = "rc4-md5"
    case CHACHA20 = "chacha20"
    case SALSA20 = "salsa20"
    case CHACHA20_IETF = "chacha20-ietf"

    // AEAD ciphers.
    case GCM_AES128 = "aes-128-gcm"
    case GCM_AES256 = "aes-256-gcm"
    case CHACHA20_IETF_POLY1305 = "chacha20-ietf-poly1305"
    case XCHACHA20_IETF_POLY1305 = "xchacha20-ietf-poly1305"
    case PMAC_SIV_AES128 = "aes-128-pmac-siv"
    case GPMAC_SIV_AES256 = "aes-256-pmac-siv"

    var description: String {
        rawValue
    }
}
