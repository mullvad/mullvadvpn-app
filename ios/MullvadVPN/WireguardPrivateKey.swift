//
//  WireguardPrivateKey.swift
//  MullvadVPN
//
//  Created by pronebird on 20/06/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import CryptoKit
import Foundation

/// A convenience wrapper around the wireguard key
struct WireguardPrivateKey {

    /// An inner impelementation of a private key
    private let innerPrivateKey: CryptoKit.Curve25519.KeyAgreement.PrivateKey

    /// Private key's raw representation
    var rawRepresentation: Data {
        return innerPrivateKey.rawRepresentation
    }

    /// Public key's raw representation
    var publicKeyRawRepresentation: Data {
        return innerPrivateKey.publicKey.rawRepresentation
    }

    /// Initialize the new private key
    init() {
        innerPrivateKey = CryptoKit.Curve25519.KeyAgreement.PrivateKey()
    }

    /// Load with the existing private key
    init(rawRepresentation: Data) throws {
        innerPrivateKey = try CryptoKit.Curve25519.KeyAgreement.PrivateKey(rawRepresentation: rawRepresentation)
    }

}

extension WireguardPrivateKey: Codable {
    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        try container.encode(innerPrivateKey.rawRepresentation)
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()

        let privateKeyBytes = try container.decode(Data.self)

        self = try .init(rawRepresentation: privateKeyBytes)
    }
}
