//
//  StoredWgKeyData.swift
//  MullvadSettings
//
//  Created by Marco Nikic on 2023-10-23.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
@preconcurrency import WireGuardKitTypes

public struct StoredWgKeyData: Codable, Equatable, Sendable {
    /// Private key creation date.
    public var creationDate: Date

    /// Last date a rotation was attempted. Nil if last attempt was successful.
    public var lastRotationAttemptDate: Date?

    /// Private key.
    public var privateKey: PrivateKey

    /// Next private key we're trying to rotate to.
    /// Added in 2023.3
    public var nextPrivateKey: PrivateKey?

    public init(
        creationDate: Date,
        lastRotationAttemptDate: Date? = nil,
        privateKey: PrivateKey,
        nextPrivateKey: PrivateKey? = nil
    ) {
        self.creationDate = creationDate
        self.lastRotationAttemptDate = lastRotationAttemptDate
        self.privateKey = privateKey
        self.nextPrivateKey = nextPrivateKey
    }
}
