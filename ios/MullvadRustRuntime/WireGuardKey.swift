//
//  WireGuardKey.swift
//  MullvadRustRuntime
//
//  Created by Emils on 2026-04-07.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

private let keyLength = 32

/// A WireGuard private key backed by a raw `Data` buffer.
///
/// Key generation and public key derivation are performed via Rust FFI
/// using talpid-types' x25519 implementation.
///
/// The `Codable` format is a single-value container encoding raw `Data`,
/// matching the format used by WireGuardKit for backward compatibility.
public struct PrivateKey: Sendable {
    public let rawValue: Data

    /// Initialize with existing raw key data.
    /// Returns `nil` if `rawValue` is not exactly 32 bytes.
    public init?(rawValue: Data) {
        guard rawValue.count == keyLength else { return nil }
        self.rawValue = rawValue
    }

    /// Generate a new random private key.
    public init() {
        var keyData = Data(repeating: 0, count: keyLength)
        keyData.withUnsafeMutableBytes { buffer in
            mullvad_generate_private_key(buffer.baseAddress!.assumingMemoryBound(to: UInt8.self))
        }
        self.rawValue = keyData
    }

    /// Derive the corresponding public key.
    public var publicKey: PublicKey {
        rawValue.withUnsafeBytes { privateBuffer in
            var publicKeyData = Data(repeating: 0, count: keyLength)
            let privateKeyBytes = privateBuffer.baseAddress!.assumingMemoryBound(to: UInt8.self)
            publicKeyData.withUnsafeMutableBytes { publicBuffer in
                mullvad_derive_public_key(
                    privateKeyBytes,
                    publicBuffer.baseAddress!.assumingMemoryBound(to: UInt8.self)
                )
            }
            return PublicKey(rawValue: publicKeyData)!
        }
    }
}

/// A WireGuard public key backed by a raw `Data` buffer.
///
/// The `Codable` format is a single-value container encoding raw `Data`,
/// matching the format used by WireGuardKit for backward compatibility.
public struct PublicKey: Sendable {
    public let rawValue: Data

    /// Initialize with existing raw key data.
    /// Returns `nil` if `rawValue` is not exactly 32 bytes.
    public init?(rawValue: Data) {
        guard rawValue.count == keyLength else { return nil }
        self.rawValue = rawValue
    }
}

/// A WireGuard pre-shared key backed by a raw `Data` buffer.
public struct PreSharedKey: Sendable {
    public let rawValue: Data

    /// Initialize with existing raw key data.
    /// Returns `nil` if `rawValue` is not exactly 32 bytes.
    public init?(rawValue: Data) {
        guard rawValue.count == keyLength else { return nil }
        self.rawValue = rawValue
    }
}

// MARK: - Codable

// Encodes/decodes as a single-value container with raw Data.
// This matches the WireGuardKit serialization format exactly.

extension PrivateKey: Codable {
    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let data = try container.decode(Data.self)
        guard let key = PrivateKey(rawValue: data) else {
            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "Invalid private key data (\(data.count) bytes, expected \(keyLength))."
            )
        }
        self = key
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(rawValue)
    }
}

extension PublicKey: Codable {
    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let data = try container.decode(Data.self)
        guard let key = PublicKey(rawValue: data) else {
            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "Invalid public key data (\(data.count) bytes, expected \(keyLength))."
            )
        }
        self = key
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(rawValue)
    }
}

extension PreSharedKey: Codable {
    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let data = try container.decode(Data.self)
        guard let key = PreSharedKey(rawValue: data) else {
            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "Invalid pre-shared key data (\(data.count) bytes, expected \(keyLength))."
            )
        }
        self = key
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(rawValue)
    }
}

// MARK: - Equatable

extension PrivateKey: Equatable {
    public static func == (lhs: PrivateKey, rhs: PrivateKey) -> Bool {
        constantTimeEquals(lhs.rawValue, rhs.rawValue)
    }
}

extension PublicKey: Equatable {
    public static func == (lhs: PublicKey, rhs: PublicKey) -> Bool {
        constantTimeEquals(lhs.rawValue, rhs.rawValue)
    }
}

extension PreSharedKey: Equatable {
    public static func == (lhs: PreSharedKey, rhs: PreSharedKey) -> Bool {
        constantTimeEquals(lhs.rawValue, rhs.rawValue)
    }
}

// MARK: - Hashable

extension PrivateKey: Hashable {
    public func hash(into hasher: inout Hasher) {
        hasher.combine(rawValue)
    }
}

extension PublicKey: Hashable {
    public func hash(into hasher: inout Hasher) {
        hasher.combine(rawValue)
    }
}

extension PreSharedKey: Hashable {
    public func hash(into hasher: inout Hasher) {
        hasher.combine(rawValue)
    }
}

// MARK: - String encoding

extension PrivateKey {
    /// Hex-encoded representation of the key.
    public var hexKey: String {
        rawValue.map { String(format: "%02x", $0) }.joined()
    }

    /// Base64-encoded representation of the key.
    public var base64Key: String {
        rawValue.base64EncodedString()
    }

    /// Initialize from a hex-encoded string.
    public init?(hexKey: String) {
        guard let data = Data(hexString: hexKey), data.count == keyLength else { return nil }
        self.rawValue = data
    }

    /// Initialize from a base64-encoded string.
    public init?(base64Key: String) {
        guard let data = Data(base64Encoded: base64Key), data.count == keyLength else { return nil }
        self.rawValue = data
    }
}

extension PublicKey {
    /// Hex-encoded representation of the key.
    public var hexKey: String {
        rawValue.map { String(format: "%02x", $0) }.joined()
    }

    /// Base64-encoded representation of the key.
    public var base64Key: String {
        rawValue.base64EncodedString()
    }

    /// Initialize from a hex-encoded string.
    public init?(hexKey: String) {
        guard let data = Data(hexString: hexKey), data.count == keyLength else { return nil }
        self.rawValue = data
    }

    /// Initialize from a base64-encoded string.
    public init?(base64Key: String) {
        guard let data = Data(base64Encoded: base64Key), data.count == keyLength else { return nil }
        self.rawValue = data
    }
}

// MARK: - RawRepresentable

extension PrivateKey: RawRepresentable {}
extension PublicKey: RawRepresentable {}
extension PreSharedKey: RawRepresentable {}

// MARK: - Helpers

/// Constant-time comparison to avoid timing side channels.
private func constantTimeEquals(_ lhs: Data, _ rhs: Data) -> Bool {
    guard lhs.count == rhs.count else { return false }
    return lhs.withUnsafeBytes { lhsBuffer in
        rhs.withUnsafeBytes { rhsBuffer in
            let lhsBytes = lhsBuffer.bindMemory(to: UInt8.self)
            let rhsBytes = rhsBuffer.bindMemory(to: UInt8.self)
            var result: UInt8 = 0
            for i in 0..<lhsBytes.count {
                result |= lhsBytes[i] ^ rhsBytes[i]
            }
            return result == 0
        }
    }
}

private extension Data {
    init?(hexString: String) {
        let chars = Array(hexString)
        guard chars.count.isMultiple(of: 2) else { return nil }
        var data = Data(capacity: chars.count / 2)
        for i in stride(from: 0, to: chars.count, by: 2) {
            guard let byte = UInt8(String(chars[i...i + 1]), radix: 16) else { return nil }
            data.append(byte)
        }
        self = data
    }
}
