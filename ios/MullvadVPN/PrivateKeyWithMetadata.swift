//
//  PrivateKeyWithMetadata.swift
//  MullvadVPN
//
//  Created by pronebird on 20/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import class WireGuardKit.PrivateKey
import class WireGuardKit.PublicKey

/// A struct holding a private WireGuard key with associated metadata
struct PrivateKeyWithMetadata: Equatable {

    /// When the key was created
    let creationDate: Date

    /// Private key
    let privateKey: PrivateKey

    /// Public key metadata
    var publicKeyWithMetadata: PublicKeyWithMetadata {
        return PublicKeyWithMetadata(publicKey: privateKey.publicKey, createdAt: creationDate)
    }

    /// Public key
    var publicKey: PublicKey {
        return privateKey.publicKey
    }

    /// Initialize the new private key
    init() {
        privateKey = PrivateKey()
        creationDate = Date()
    }

    /// Initialize with the existing private key
    init(privateKey: PrivateKey, createdAt: Date) {
        self.privateKey = privateKey
        creationDate = createdAt
    }

}

/// A struct holding a public WireGuard key with associated metadata
struct PublicKeyWithMetadata: Equatable {
    /// Refers to private key creation date
    let creationDate: Date

    /// Public key
    let publicKey: PublicKey

    init(publicKey: PublicKey, createdAt: Date) {
        self.publicKey = publicKey
        creationDate = createdAt
    }

    /// Returns a base64 encoded string representation that can be used for displaying the key in
    /// the user interface
    func stringRepresentation(maxLength: Int? = nil) -> String {
        let base64EncodedKey = publicKey.base64Key

        if let maxLength = maxLength, maxLength < base64EncodedKey.count {
            return base64EncodedKey.prefix(maxLength) + "..."
        } else {
            return base64EncodedKey
        }
    }
}

extension PrivateKeyWithMetadata: Codable {

    private enum CodingKeys: String, CodingKey {
        case privateKeyData, creationDate
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        try container.encode(privateKey.rawValue, forKey: .privateKeyData)
        try container.encode(creationDate, forKey: .creationDate)
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let privateKeyBytes = try container.decode(Data.self, forKey: .privateKeyData)

        guard let privateKey = PrivateKey(rawValue: privateKeyBytes) else {
            throw DecodingError.dataCorruptedError(
                forKey: CodingKeys.privateKeyData,
                in: container,
                debugDescription: "Invalid key data"
            )
        }

        self.privateKey = privateKey
        self.creationDate = try container.decode(Date.self, forKey: .creationDate)
    }
}
