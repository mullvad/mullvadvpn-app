// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadSettings
import MullvadTypes
import XCTest

final class WgKeyRotationTests: XCTestCase {
    func testKeyRotationLifecycle() {
        let data = StoredDeviceData.mock(
            keyData: StoredWgKeyData(
                creationDate: Date(),
                privateKey: WireGuard.PrivateKey()
            )
        )

        var keyRotation = WgKeyRotation(data: data)
        let nextPubKey = keyRotation.beginAttempt()

        let nextKey = keyRotation.data.wgKeyData.nextPrivateKey
        let lastRotationDate = keyRotation.data.wgKeyData.lastRotationAttemptDate

        XCTAssertNotNil(nextKey)
        XCTAssertNotNil(lastRotationDate)
        XCTAssertEqual(nextPubKey, nextKey?.publicKey)

        XCTAssertTrue(keyRotation.setCompleted(with: Device.mock(privateKey: nextKey!)))

        XCTAssertNil(keyRotation.data.wgKeyData.lastRotationAttemptDate)
        XCTAssertNil(keyRotation.data.wgKeyData.nextPrivateKey)
        XCTAssertEqual(keyRotation.data.wgKeyData.privateKey, nextKey)
    }

    func testHandlesMultipleKeyRotationAttempts() {
        let currentKey = WireGuard.PrivateKey()
        let nextKey = WireGuard.PrivateKey()
        let data = StoredDeviceData.mock(
            keyData: StoredWgKeyData(
                creationDate: Date(),
                lastRotationAttemptDate: Date(),
                privateKey: currentKey,
                nextPrivateKey: nextKey
            )
        )

        var keyRotation = WgKeyRotation(data: data)
        let pubKey = keyRotation.beginAttempt()
        let lastAttemptDate = keyRotation.data.wgKeyData.lastRotationAttemptDate

        let samePubKey = keyRotation.beginAttempt()
        let anotherAttemptDate = keyRotation.data.wgKeyData.lastRotationAttemptDate

        XCTAssertEqual(pubKey, nextKey.publicKey)
        XCTAssertEqual(pubKey, samePubKey)
        XCTAssertNotEqual(lastAttemptDate, anotherAttemptDate)

        XCTAssertEqual(keyRotation.data.wgKeyData.privateKey, currentKey)
        XCTAssertEqual(keyRotation.data.wgKeyData.nextPrivateKey, nextKey)
    }

    func testHandlesMultipleKeyRotationCompletions() {
        let currentKey = WireGuard.PrivateKey()
        let nextKey = WireGuard.PrivateKey()
        let data = StoredDeviceData.mock(
            keyData: StoredWgKeyData(
                creationDate: Date(),
                lastRotationAttemptDate: Date(),
                privateKey: currentKey,
                nextPrivateKey: nextKey
            )
        )

        var keyRotation = WgKeyRotation(data: data)

        XCTAssertTrue(keyRotation.setCompleted(with: Device.mock(privateKey: nextKey)))
        XCTAssertFalse(keyRotation.setCompleted(with: Device.mock(privateKey: nextKey)))

        XCTAssertEqual(keyRotation.data.wgKeyData.privateKey, nextKey)
        XCTAssertNil(keyRotation.data.wgKeyData.nextPrivateKey)
        XCTAssertNil(keyRotation.data.wgKeyData.lastRotationAttemptDate)
    }
}

private extension StoredDeviceData {
    static func mock(keyData: StoredWgKeyData) -> StoredDeviceData {
        StoredDeviceData(
            creationDate: Date(),
            identifier: "device-id",
            name: "device-name",
            hijackDNS: false,
            ipv4Address: IPAddressRange(from: "127.0.0.1/32")!,
            ipv6Address: IPAddressRange(from: "::ff/64")!,
            wgKeyData: keyData
        )
    }
}

private extension Device {
    static func mock(privateKey: WireGuard.PrivateKey) -> Device {
        Device(
            id: "device-id",
            name: "device-name",
            pubkey: privateKey.publicKey,
            hijackDNS: false,
            created: Date(),
            ipv4Address: IPAddressRange(from: "127.0.0.1/32")!,
            ipv6Address: IPAddressRange(from: "::ff/64")!
        )
    }
}
