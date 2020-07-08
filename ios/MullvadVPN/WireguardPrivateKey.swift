//
//  WireguardPrivateKey.swift
//  MullvadVPN
//
//  Created by pronebird on 20/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// A convenience wrapper around the wireguard key
struct WireguardPrivateKey {

    /// When the key was created
    let creationDate: Date

    /// Private key's raw representation
    private(set) var rawRepresentation: Data

    /// Public key
    var publicKey: WireguardPublicKey {
        WireguardPublicKey(
            creationDate: creationDate,
            rawRepresentation: Curve25519.generatePublicKey(fromPrivateKey: rawRepresentation)
        )
    }

    /// Initialize the new private key
    init() {
        rawRepresentation = Curve25519.generatePrivateKey()
        creationDate = Date()
    }

    /// Load with the existing private key
    init?(rawRepresentation: Data, createdAt: Date) {
        guard rawRepresentation.count == Curve25519.keyLength else { return nil }

        self.rawRepresentation = rawRepresentation
        creationDate = createdAt
    }

}

extension WireguardPrivateKey: Equatable {
    static func == (lhs: WireguardPrivateKey, rhs: WireguardPrivateKey) -> Bool {
        lhs.rawRepresentation == rhs.rawRepresentation
    }
}

/// A struct holding a public key used for Wireguard with associated metadata
struct WireguardPublicKey: Codable, Equatable {
    /// Refers to private key creation date
    let creationDate: Date

    /// Raw public key representation
    let rawRepresentation: Data

    /// Returns a base64 encoded string representation that can be used for displaying the key in
    /// the user interface
    func stringRepresentation(maxLength: Int? = nil) -> String {
        let base64EncodedKey = rawRepresentation.base64EncodedString()

        if let maxLength = maxLength, maxLength < base64EncodedKey.count {
            return base64EncodedKey.prefix(maxLength) + "..."
        } else {
            return base64EncodedKey
        }
    }
}

extension WireguardPrivateKey: Codable {

    private enum CodingKeys: String, CodingKey {
        case privateKeyData, creationDate
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        try container.encode(rawRepresentation, forKey: .privateKeyData)
        try container.encode(creationDate, forKey: .creationDate)
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let privateKeyBytes = try container.decode(Data.self, forKey: .privateKeyData)
        let creationDate = try container.decode(Date.self, forKey: .creationDate)
        
        if let instance = WireguardPrivateKey(rawRepresentation: privateKeyBytes, createdAt: creationDate) {
            self = instance
        } else {
            throw DecodingError.dataCorruptedError(
                forKey: CodingKeys.privateKeyData,
                in: container,
                debugDescription: "Invalid key data"
            )
        }
    }
}
