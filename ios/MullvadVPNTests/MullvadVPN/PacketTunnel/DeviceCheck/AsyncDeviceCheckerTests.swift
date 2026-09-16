// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadREST
import MullvadSettings
import MullvadTypes
import PacketTunnelCore
import XCTest

@testable import MullvadMockData

class AsyncDeviceCheckerTests: XCTestCase {
    func testShouldReportExpiredAccount() async throws {
        let currentKey = WireGuard.PrivateKey()
        let remoteService = MockRemoteService(
            initialKey: currentKey.publicKey,
            getAccount: { _ in
                Account.mock(expiry: .distantPast)
            }
        )
        let deviceStateAccessor = MockDeviceStateAccessor.mockLoggedIn(
            currentKey: currentKey,
            rotationState: .succeeded
        )

        let deviceCheck = try await check(remoteService: remoteService, deviceStateAccessor: deviceStateAccessor).get()

        XCTAssertTrue(deviceCheck.accountVerdict.isExpired)
        XCTAssertEqual(deviceCheck.keyRotationStatus, .noAction)
        XCTAssertEqual(try deviceStateAccessor.read().deviceData?.wgKeyData.privateKey, currentKey)
    }

    func testShouldNotRotateKeyForInvalidAccount() async throws {
        let currentKey = WireGuard.PrivateKey()
        let nextKey = WireGuard.PrivateKey()

        let remoteService = MockRemoteService(
            initialKey: currentKey.publicKey,
            getAccount: { _ in
                throw REST.Error.unhandledResponse(404, REST.ServerErrorResponse(code: .invalidAccount))
            }
        )
        let deviceStateAccessor = MockDeviceStateAccessor.mockLoggedIn(
            currentKey: currentKey,
            rotationState: .failed(when: .retryInterval, nextKey: nextKey)
        )

        let deviceCheck = try await check(remoteService: remoteService, deviceStateAccessor: deviceStateAccessor).get()

        XCTAssertEqual(deviceCheck.accountVerdict, .invalid)
        XCTAssertEqual(deviceCheck.keyRotationStatus, .noAction)
        XCTAssertEqual(try deviceStateAccessor.read().deviceData?.wgKeyData.privateKey, currentKey)
    }

    func testShouldNotRotateKeyForRevokedDevice() async throws {
        let currentKey = WireGuard.PrivateKey()
        let nextKey = WireGuard.PrivateKey()

        let remoteService = MockRemoteService(
            initialKey: currentKey.publicKey,
            getDevice: { _, _ in
                throw REST.Error.unhandledResponse(404, REST.ServerErrorResponse(code: .deviceNotFound))
            }
        )
        let deviceStateAccessor = MockDeviceStateAccessor.mockLoggedIn(
            currentKey: currentKey,
            rotationState: .failed(when: .retryInterval, nextKey: nextKey)
        )

        let deviceCheck = try await check(remoteService: remoteService, deviceStateAccessor: deviceStateAccessor).get()

        XCTAssertEqual(deviceCheck.deviceVerdict, .revoked)
        XCTAssertEqual(deviceCheck.keyRotationStatus, .noAction)
        XCTAssertEqual(try deviceStateAccessor.read().deviceData?.wgKeyData.privateKey, currentKey)
    }

    func testShouldRotateKeyOnMismatchImmediately() async throws {
        let currentKey = WireGuard.PrivateKey()
        let nextKey = WireGuard.PrivateKey()

        let remoteService = MockRemoteService()
        let deviceStateAccessor = MockDeviceStateAccessor.mockLoggedIn(
            currentKey: currentKey,
            rotationState: .failed(when: .packetTunnelCooldownInterval, nextKey: nextKey)
        )

        let deviceCheck = try await check(
            remoteService: remoteService,
            deviceStateAccessor: deviceStateAccessor,
            rotateImmediatelyOnKeyMismatch: true
        ).get()

        XCTAssertTrue(deviceCheck.keyRotationStatus.isSucceeded)
        XCTAssertEqual(deviceCheck.deviceVerdict, .active)
        XCTAssertEqual(try deviceStateAccessor.read().deviceData?.wgKeyData.privateKey, nextKey)
    }

    func testShouldRespectCooldownWhenRotatingImmediately() async throws {
        let currentKey = WireGuard.PrivateKey()
        let nextKey = WireGuard.PrivateKey()

        let remoteService = MockRemoteService()
        let deviceStateAccessor = MockDeviceStateAccessor.mockLoggedIn(
            currentKey: currentKey,
            rotationState: .failed(when: .zero, nextKey: nextKey)
        )

        let deviceCheck = try await check(
            remoteService: remoteService,
            deviceStateAccessor: deviceStateAccessor,
            rotateImmediatelyOnKeyMismatch: true
        ).get()

        XCTAssertEqual(deviceCheck.keyRotationStatus, .noAction)
        XCTAssertEqual(deviceCheck.deviceVerdict, .keyMismatch)
        XCTAssertEqual(try deviceStateAccessor.read().deviceData?.wgKeyData.privateKey, currentKey)
    }

    func testShouldNotRotateDeviceKeyWhenServerKeyIsIdentical() async throws {
        let currentKey = WireGuard.PrivateKey()
        let remoteService = MockRemoteService(initialKey: currentKey.publicKey)
        let deviceStateAccessor = MockDeviceStateAccessor.mockLoggedIn(
            currentKey: currentKey,
            rotationState: .succeeded
        )

        let deviceCheck = try await check(remoteService: remoteService, deviceStateAccessor: deviceStateAccessor).get()

        XCTAssertEqual(deviceCheck.deviceVerdict, .active)
        XCTAssertEqual(deviceCheck.keyRotationStatus, .noAction)
        XCTAssertEqual(try deviceStateAccessor.read().deviceData?.wgKeyData.privateKey, currentKey)
    }

    func testShouldNotRotateKeyBeforeRetryIntervalPassed() async throws {
        let currentKey = WireGuard.PrivateKey()
        let nextKey = WireGuard.PrivateKey()

        let remoteService = MockRemoteService(initialKey: currentKey.publicKey)
        let deviceStateAccessor = MockDeviceStateAccessor.mockLoggedIn(
            currentKey: currentKey,
            rotationState: .failed(when: .closeToRetryInterval, nextKey: nextKey)
        )

        let deviceCheck = try await check(remoteService: remoteService, deviceStateAccessor: deviceStateAccessor).get()

        XCTAssertEqual(deviceCheck.keyRotationStatus, .noAction)
        XCTAssertEqual(try deviceStateAccessor.read().deviceData?.wgKeyData.privateKey, currentKey)
    }

    func testShouldRotateKeyOnceInTwentyFourHours() async throws {
        let currentKey = WireGuard.PrivateKey()
        let nextKey = WireGuard.PrivateKey()

        let remoteService = MockRemoteService()
        let deviceStateAccessor = MockDeviceStateAccessor.mockLoggedIn(
            currentKey: currentKey,
            rotationState: .failed(when: .retryInterval, nextKey: nextKey)
        )

        let deviceCheck = try await check(remoteService: remoteService, deviceStateAccessor: deviceStateAccessor).get()

        XCTAssertTrue(deviceCheck.keyRotationStatus.isSucceeded)
        XCTAssertEqual(deviceCheck.deviceVerdict, .active)

        let keyData = try XCTUnwrap(deviceStateAccessor.read().deviceData?.wgKeyData)
        XCTAssertEqual(keyData.privateKey, nextKey)
        XCTAssertNil(keyData.nextPrivateKey)
        XCTAssertNil(keyData.lastRotationAttemptDate)
    }

    func testShouldReportFailedKeyRotationAttempt() async throws {
        let currentKey = WireGuard.PrivateKey()
        let nextKey = WireGuard.PrivateKey()

        let remoteService = MockRemoteService(
            rotateDeviceKey: { _, _, _ in
                throw URLError(.badURL)
            }
        )
        let deviceStateAccessor = MockDeviceStateAccessor.mockLoggedIn(
            currentKey: currentKey,
            rotationState: .failed(when: .retryInterval, nextKey: nextKey)
        )

        let deviceCheck = try await check(remoteService: remoteService, deviceStateAccessor: deviceStateAccessor).get()

        XCTAssertTrue(deviceCheck.keyRotationStatus.isAttempted)
        XCTAssertEqual(deviceCheck.deviceVerdict, .keyMismatch)

        // The attempt is persisted before the request so the next attempt reuses the key and honours the interval.
        let keyData = try XCTUnwrap(deviceStateAccessor.read().deviceData?.wgKeyData)
        XCTAssertEqual(keyData.privateKey, currentKey)
        XCTAssertEqual(keyData.nextPrivateKey, nextKey)
        XCTAssertEqual(keyData.lastRotationAttemptDate.map { Date().timeIntervalSince($0) < 5 }, true)
    }

    func testShouldPushSameKeyOnRetry() async throws {
        let currentKey = WireGuard.PrivateKey()
        let nextKey = WireGuard.PrivateKey()

        let remoteService = MockRemoteService(
            rotateDeviceKey: { _, _, publicKey in
                XCTAssertEqual(publicKey, nextKey.publicKey)
            }
        )
        let deviceStateAccessor = MockDeviceStateAccessor.mockLoggedIn(
            currentKey: currentKey,
            rotationState: .failed(when: .retryInterval, nextKey: nextKey)
        )

        let deviceCheck = try await check(remoteService: remoteService, deviceStateAccessor: deviceStateAccessor).get()

        XCTAssertTrue(deviceCheck.keyRotationStatus.isSucceeded)
    }

    func testShouldReportAttemptOnKeyRotationRace() async throws {
        let currentKey = WireGuard.PrivateKey()
        let nextKey = WireGuard.PrivateKey()

        let deviceStateAccessor = MockDeviceStateAccessor.mockLoggedIn(
            currentKey: currentKey,
            rotationState: .failed(when: .retryInterval, nextKey: nextKey)
        )

        let remoteService = MockRemoteService(
            rotateDeviceKey: { _, _, _ in
                // Overwrite device state before returning the result from key rotation to simulate the race condition
                // in the underlying storage.
                try deviceStateAccessor.write(
                    .loggedIn(
                        StoredAccountData.mock(),
                        StoredDeviceData.mock(wgKeyData: StoredWgKeyData(creationDate: Date(), privateKey: currentKey))
                    )
                )
            }
        )

        let deviceCheck = try await check(remoteService: remoteService, deviceStateAccessor: deviceStateAccessor).get()

        XCTAssertTrue(deviceCheck.keyRotationStatus.isAttempted)
        XCTAssertEqual(try deviceStateAccessor.read().deviceData?.wgKeyData.privateKey, currentKey)
    }

    func testShouldFailWhenLoggedOut() async throws {
        let remoteService = MockRemoteService()
        let deviceStateAccessor = MockDeviceStateAccessor(initialState: .loggedOut)

        let result = await check(remoteService: remoteService, deviceStateAccessor: deviceStateAccessor)

        XCTAssertThrowsError(try result.get()) { error in
            XCTAssertEqual(error as? DeviceCheckError, .invalidDeviceState)
        }
    }

    // MARK: - Outcome mapping

    func testOutcomeIsNoActionWhenDeviceIsActive() async {
        let currentKey = WireGuard.PrivateKey()
        let checker = makeChecker(
            remoteService: MockRemoteService(initialKey: currentKey.publicKey),
            deviceStateAccessor: MockDeviceStateAccessor.mockLoggedIn(currentKey: currentKey, rotationState: .succeeded)
        )

        let outcome = await checker.checkDevice(rotateKeyOnMismatch: false)

        XCTAssertEqual(outcome, .noAction)
    }

    func testOutcomeIsNoActionOnNetworkError() async {
        let currentKey = WireGuard.PrivateKey()
        let checker = makeChecker(
            remoteService: MockRemoteService(
                initialKey: currentKey.publicKey,
                getAccount: { _ in throw URLError(.notConnectedToInternet) }
            ),
            deviceStateAccessor: MockDeviceStateAccessor.mockLoggedIn(currentKey: currentKey, rotationState: .succeeded)
        )

        let outcome = await checker.checkDevice(rotateKeyOnMismatch: false)

        XCTAssertEqual(outcome, .noAction)
    }

    func testOutcomeIsBlockedForRevokedDevice() async {
        let currentKey = WireGuard.PrivateKey()
        let checker = makeChecker(
            remoteService: MockRemoteService(
                initialKey: currentKey.publicKey,
                getDevice: { _, _ in
                    throw REST.Error.unhandledResponse(404, REST.ServerErrorResponse(code: .deviceNotFound))
                }
            ),
            deviceStateAccessor: MockDeviceStateAccessor.mockLoggedIn(currentKey: currentKey, rotationState: .succeeded)
        )

        let outcome = await checker.checkDevice(rotateKeyOnMismatch: false)

        XCTAssertEqual(outcome, .blocked(.deviceRevoked))
    }

    func testOutcomeIsBlockedForExpiredAccount() async {
        let currentKey = WireGuard.PrivateKey()
        let checker = makeChecker(
            remoteService: MockRemoteService(
                initialKey: currentKey.publicKey,
                getAccount: { _ in Account.mock(expiry: .distantPast) }
            ),
            deviceStateAccessor: MockDeviceStateAccessor.mockLoggedIn(currentKey: currentKey, rotationState: .succeeded)
        )

        let outcome = await checker.checkDevice(rotateKeyOnMismatch: false)

        XCTAssertEqual(outcome, .blocked(.accountExpired))
    }

    func testOutcomeIsBlockedForInvalidAccount() async {
        let currentKey = WireGuard.PrivateKey()
        let checker = makeChecker(
            remoteService: MockRemoteService(
                initialKey: currentKey.publicKey,
                getAccount: { _ in
                    throw REST.Error.unhandledResponse(404, REST.ServerErrorResponse(code: .invalidAccount))
                }
            ),
            deviceStateAccessor: MockDeviceStateAccessor.mockLoggedIn(currentKey: currentKey, rotationState: .succeeded)
        )

        let outcome = await checker.checkDevice(rotateKeyOnMismatch: false)

        XCTAssertEqual(outcome, .blocked(.invalidAccount))
    }

    func testOutcomeIsBlockedWhenLoggedOut() async {
        let checker = makeChecker(
            remoteService: MockRemoteService(),
            deviceStateAccessor: MockDeviceStateAccessor(initialState: .loggedOut)
        )

        let outcome = await checker.checkDevice(rotateKeyOnMismatch: false)

        XCTAssertEqual(outcome, .blocked(.deviceLoggedOut))
    }

    func testOutcomeIsKeyRotationAfterRotating() async throws {
        let currentKey = WireGuard.PrivateKey()
        let nextKey = WireGuard.PrivateKey()
        let checker = makeChecker(
            remoteService: MockRemoteService(),
            deviceStateAccessor: MockDeviceStateAccessor.mockLoggedIn(
                currentKey: currentKey,
                rotationState: .failed(when: .retryInterval, nextKey: nextKey)
            )
        )

        let outcome = await checker.checkDevice(rotateKeyOnMismatch: false)

        guard case .keyRotation = outcome else {
            return XCTFail("Expected key rotation outcome, got \(outcome)")
        }
    }

    func testOutcomeIsKeyRotationAfterFailedAttempt() async throws {
        let currentKey = WireGuard.PrivateKey()
        let nextKey = WireGuard.PrivateKey()
        let checker = makeChecker(
            remoteService: MockRemoteService(rotateDeviceKey: { _, _, _ in throw URLError(.badURL) }),
            deviceStateAccessor: MockDeviceStateAccessor.mockLoggedIn(
                currentKey: currentKey,
                rotationState: .failed(when: .retryInterval, nextKey: nextKey)
            )
        )

        let outcome = await checker.checkDevice(rotateKeyOnMismatch: false)

        guard case .keyRotation = outcome else {
            return XCTFail("Expected key rotation outcome so the app reloads device state, got \(outcome)")
        }
    }

    // MARK: - Helpers

    private func makeChecker(
        remoteService: DeviceCheckRemoteServiceProtocol,
        deviceStateAccessor: DeviceStateAccessorProtocol
    ) -> AsyncDeviceChecker {
        AsyncDeviceChecker(remoteService: remoteService, deviceStateAccessor: deviceStateAccessor)
    }

    private func check(
        remoteService: DeviceCheckRemoteServiceProtocol,
        deviceStateAccessor: DeviceStateAccessorProtocol,
        rotateImmediatelyOnKeyMismatch: Bool = false
    ) async -> Result<DeviceCheck, Error> {
        await makeChecker(remoteService: remoteService, deviceStateAccessor: deviceStateAccessor)
            .check(rotateImmediatelyOnKeyMismatch: rotateImmediatelyOnKeyMismatch)
    }
}
