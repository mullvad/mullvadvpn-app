//
//  KeychainItemRevision.swift
//  MullvadVPN
//
//  Created by pronebird on 07/05/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// A struct that helps to organize revisions for Keychain items
/// Uses `Keychain.Attributes.generic` field for storing `Int64` revision number.
struct KeychainItemRevision {
    let revision: Int64

    /// Returns the `KeychainItemRevision` initialized with the next in sequence revision
    var nextRevision: KeychainItemRevision {
        return .init(revision: revision + 1)
    }

    /// Initialize a struct with the given revision number
    init(revision: Int64) {
        self.revision = revision
    }

    /// Initialize a struct from the given `Keychain.Attributes`
    ///
    /// Returns `nil` when `Keychain.Attributes.generic` is `nil` or when it's set to data that
    /// cannot be interpreted as `Int64`
    init?(attributes: Keychain.Attributes) {
        let revision = attributes.generic.flatMap { Self.parseRevision(from: $0) }
        if let revision = revision {
            self.revision = revision
        } else {
            return nil
        }
    }

    /// Store the revision number in the given `Keychain.Attributes`
    func store(in attributes: inout Keychain.Attributes) {
        attributes.generic = asData()
    }

    /// A convenience method to initialize the first revision in sequence
    static func firstRevision() -> Self {
        return .init(revision: 1)
    }

    /// Serialize the revision number as `Data`
    private func asData() -> Data {
        return withUnsafeBytes(of: revision) { Data($0) }
    }

    /// Private helper to parse `Data` into `Int64`
    /// Returns `nil` if the given raw data does not match the size of `Int64`
    private static func parseRevision(from rawData: Data) -> Int64? {
        var value: Int64 = 0

        // Ensure that the buffer has as many bytes as needed to fill the `Int64`
        guard rawData.count == MemoryLayout.size(ofValue: value) else {
            return nil
        }

        _ = withUnsafeMutableBytes(of: &value) { (valuePointer) in
            rawData.copyBytes(to: valuePointer)
        }

        return value
    }
}
