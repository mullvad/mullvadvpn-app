//
//  WireguardPrivateKey.swift
//  MullvadVPN
//
//  Created by pronebird on 20/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import CryptoKit
import Foundation

/// A convenience wrapper around the wireguard key
struct WireguardPrivateKey {

    /// When the key was created
    let creationDate: Date

    /// Private key's raw representation
    var rawRepresentation: Data {
        innerPrivateKey.rawRepresentation
    }

    /// Public key
    var publicKey: WireguardPublicKey {
        WireguardPublicKey(
            creationDate: creationDate,
            rawRepresentation: innerPrivateKey.publicKey.rawRepresentation
        )
    }

    /// An inner impelementation of a private key
    private let innerPrivateKey: Curve25519.KeyAgreement.PrivateKey

    /// Initialize the new private key
    init() {
        innerPrivateKey = Curve25519.KeyAgreement.PrivateKey()
        creationDate = Date()
    }

    /// Load with the existing private key
    init(rawRepresentation: Data, createdAt: Date) throws {
        innerPrivateKey = try Curve25519.KeyAgreement.PrivateKey(rawRepresentation: rawRepresentation)
        creationDate = createdAt
    }

}

extension WireguardPrivateKey: Equatable {
    static func == (lhs: WireguardPrivateKey, rhs: WireguardPrivateKey) -> Bool {
        lhs.rawRepresentation == rhs.rawRepresentation
    }
}

/// A struct holding a public key used for Wireguard with associated metadata
struct WireguardPublicKey {
    /// Refers to private key creation date
    let creationDate: Date

    /// Raw public key representation
    let rawRepresentation: Data
}

extension WireguardPrivateKey: Codable {

    private enum CodingKeys: String, CodingKey {
        case privateKeyData, creationDate
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        try container.encode(innerPrivateKey.rawRepresentation, forKey: .privateKeyData)
        try container.encode(creationDate, forKey: .creationDate)
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let privateKeyBytes = try container.decode(Data.self, forKey: .privateKeyData)
        let creationDate = try container.decode(Date.self, forKey: .creationDate)

        self = try .init(rawRepresentation: privateKeyBytes, createdAt: creationDate)
    }
}
