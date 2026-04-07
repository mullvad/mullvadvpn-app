//
//  WireGuardKeyTests.swift
//  MullvadRustRuntimeTests
//
//  Created by Emils on 2026-04-07.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import XCTest

@testable import MullvadRustRuntime
@testable import MullvadTypes
@testable import WireGuardKitTypes

// Namespace the new types so they don't collide with WireGuardKitTypes in this test file.
typealias NewPrivateKey = WireGuard.PrivateKey
typealias NewPublicKey = WireGuard.PublicKey
typealias NewPreSharedKey = WireGuard.PreSharedKey
typealias OldPrivateKey = WireGuardKitTypes.PrivateKey
typealias OldPublicKey = WireGuardKitTypes.PublicKey
typealias OldPreSharedKey = WireGuardKitTypes.PreSharedKey

class WireGuardKeyTests: XCTestCase {

    // MARK: - Golden deserialization: old PrivateKey -> new PrivateKey

    func testDeserializeOldPrivateKeyWithNewType() throws {
        // Serialize a key using the old WireGuardKit type
        let oldKey = OldPrivateKey()
        let encodedData = try JSONEncoder().encode(oldKey)

        // Deserialize with the new type
        let newKey = try JSONDecoder().decode(NewPrivateKey.self, from: encodedData)

        // Raw bytes must match exactly
        XCTAssertEqual(oldKey.rawValue, newKey.rawValue)
    }

    // MARK: - Golden deserialization: old PublicKey -> new PublicKey

    func testDeserializeOldPublicKeyWithNewType() throws {
        let oldKey = OldPrivateKey().publicKey
        let encodedData = try JSONEncoder().encode(oldKey)

        let newKey = try JSONDecoder().decode(NewPublicKey.self, from: encodedData)

        XCTAssertEqual(oldKey.rawValue, newKey.rawValue)
    }

    // MARK: - Golden deserialization: old PreSharedKey -> new PreSharedKey

    func testDeserializeOldPreSharedKeyWithNewType() throws {
        let rawData = Data((0..<32).map { _ in UInt8.random(in: 0...255) })
        let oldKey = OldPreSharedKey(rawValue: rawData)!
        let encodedData = try JSONEncoder().encode(oldKey)

        let newKey = try JSONDecoder().decode(NewPreSharedKey.self, from: encodedData)

        XCTAssertEqual(oldKey.rawValue, newKey.rawValue)
    }

    // MARK: - Round-trip: new type serialization

    func testPrivateKeyRoundTrip() throws {
        let key = NewPrivateKey()
        let encodedData = try JSONEncoder().encode(key)
        let decodedKey = try JSONDecoder().decode(NewPrivateKey.self, from: encodedData)

        XCTAssertEqual(key, decodedKey)
    }

    func testPublicKeyRoundTrip() throws {
        let key = NewPrivateKey().publicKey
        let encodedData = try JSONEncoder().encode(key)
        let decodedKey = try JSONDecoder().decode(NewPublicKey.self, from: encodedData)

        XCTAssertEqual(key, decodedKey)
    }

    // MARK: - Cross-compatibility: new serialized -> old deserialized

    func testNewPrivateKeyDeserializableByOldType() throws {
        let newKey = NewPrivateKey()
        let encodedData = try JSONEncoder().encode(newKey)

        let oldKey = try JSONDecoder().decode(OldPrivateKey.self, from: encodedData)

        XCTAssertEqual(newKey.rawValue, oldKey.rawValue)
    }

    func testNewPublicKeyDeserializableByOldType() throws {
        let newKey = NewPrivateKey().publicKey
        let encodedData = try JSONEncoder().encode(newKey)

        let oldKey = try JSONDecoder().decode(OldPublicKey.self, from: encodedData)

        XCTAssertEqual(newKey.rawValue, oldKey.rawValue)
    }

    // MARK: - Key derivation consistency

    func testPublicKeyDerivationMatchesOldImplementation() throws {
        // Use the same raw bytes for both old and new private keys
        let rawKeyData = Data((0..<32).map { _ in UInt8.random(in: 0...255) })

        let oldKey = OldPrivateKey(rawValue: rawKeyData)!
        let newKey = NewPrivateKey(rawValue: rawKeyData)!

        // Derived public keys must be identical
        XCTAssertEqual(oldKey.publicKey.rawValue, newKey.publicKey.rawValue)
    }

    // MARK: - String encoding

    func testBase64KeyMatchesOldImplementation() throws {
        let rawKeyData = Data((0..<32).map { _ in UInt8.random(in: 0...255) })

        let oldKey = OldPublicKey(rawValue: rawKeyData)!
        let newKey = NewPublicKey(rawValue: rawKeyData)!

        XCTAssertEqual(oldKey.base64Key, newKey.base64Key)
    }

    func testHexKeyMatchesOldImplementation() throws {
        let rawKeyData = Data((0..<32).map { _ in UInt8.random(in: 0...255) })

        let oldKey = OldPublicKey(rawValue: rawKeyData)!
        let newKey = NewPublicKey(rawValue: rawKeyData)!

        XCTAssertEqual(oldKey.hexKey, newKey.hexKey)
    }

    // MARK: - Init from string

    func testInitFromBase64() throws {
        let key = NewPrivateKey()
        let base64 = key.base64Key

        let restored = NewPrivateKey(base64Key: base64)
        XCTAssertNotNil(restored)
        XCTAssertEqual(key, restored)
    }

    func testInitFromHex() throws {
        let key = NewPrivateKey()
        let hex = key.hexKey

        let restored = NewPrivateKey(hexKey: hex)
        XCTAssertNotNil(restored)
        XCTAssertEqual(key, restored)
    }

    // MARK: - Validation

    func testRejectsInvalidKeyLength() {
        let tooShort = Data(repeating: 0, count: 16)
        let tooLong = Data(repeating: 0, count: 64)

        XCTAssertNil(NewPrivateKey(rawValue: tooShort))
        XCTAssertNil(NewPrivateKey(rawValue: tooLong))
        XCTAssertNil(NewPublicKey(rawValue: tooShort))
        XCTAssertNil(NewPublicKey(rawValue: tooLong))
        XCTAssertNil(NewPreSharedKey(rawValue: tooShort))
        XCTAssertNil(NewPreSharedKey(rawValue: tooLong))
    }

    func testDecodingInvalidDataFails() {
        let invalidJSON = try! JSONEncoder().encode(Data(repeating: 0, count: 16))

        XCTAssertThrowsError(try JSONDecoder().decode(NewPrivateKey.self, from: invalidJSON))
        XCTAssertThrowsError(try JSONDecoder().decode(NewPublicKey.self, from: invalidJSON))
        XCTAssertThrowsError(try JSONDecoder().decode(NewPreSharedKey.self, from: invalidJSON))
    }
}
