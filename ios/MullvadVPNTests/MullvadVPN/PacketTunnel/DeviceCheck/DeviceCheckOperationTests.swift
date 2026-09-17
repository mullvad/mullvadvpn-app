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
import Operations
import PacketTunnelCore
import XCTest

@testable import MullvadMockData

class DeviceCheckOperationTests: XCTestCase {
    private let operationQueue = AsyncOperationQueue()
    private let dispatchQueue = DispatchQueue(label: "TestQueue")

    func testShouldReportExpiredAccount() async {
        let expect = expectation(description: "Wait for operation to complete")

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

        startDeviceCheck(remoteService: remoteService, deviceStateAccessor: deviceStateAccessor) { result in
            let deviceCheck = result.value

            XCTAssertNotNil(deviceCheck)
            XCTAssertTrue(deviceCheck?.accountVerdict.isExpired ?? false)
            XCTAssert(deviceCheck?.keyRotationStatus == .noAction)
            XCTAssertEqual(try? deviceStateAccessor.read().deviceData?.wgKeyData.privateKey, currentKey)

            expect.fulfill()
        }

        await fulfillment(of: [expect], timeout: .UnitTest.timeout)
    }

    func testShouldNotRotateKeyForInvalidAccount() async {
        let expect = expectation(description: "Wait for operation to complete")

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

        startDeviceCheck(remoteService: remoteService, deviceStateAccessor: deviceStateAccessor) { result in
            let deviceCheck = result.value

            XCTAssertNotNil(deviceCheck)
            XCTAssert(deviceCheck?.accountVerdict == .invalid)
            XCTAssert(deviceCheck?.keyRotationStatus == .noAction)
            XCTAssertEqual(try? deviceStateAccessor.read().deviceData?.wgKeyData.privateKey, currentKey)

            expect.fulfill()
        }

        await fulfillment(of: [expect], timeout: .UnitTest.timeout)
    }

    func testShouldNotRotateKeyForRevokedDevice() async {
        let expect = expectation(description: "Wait for operation to complete")

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

        startDeviceCheck(remoteService: remoteService, deviceStateAccessor: deviceStateAccessor) { result in
            let deviceCheck = result.value

            XCTAssertNotNil(deviceCheck)
            XCTAssert(deviceCheck?.deviceVerdict == .revoked)
            XCTAssert(deviceCheck?.keyRotationStatus == .noAction)
            XCTAssertEqual(try? deviceStateAccessor.read().deviceData?.wgKeyData.privateKey, currentKey)

            expect.fulfill()
        }

        await fulfillment(of: [expect], timeout: .UnitTest.timeout)
    }

    func testShouldRotateKeyOnMismatchImmediately() async {
        let expect = expectation(description: "Wait for operation to complete")

        let currentKey = WireGuard.PrivateKey()
        let nextKey = WireGuard.PrivateKey()

        let remoteService = MockRemoteService()
        let deviceStateAccessor = MockDeviceStateAccessor.mockLoggedIn(
            currentKey: currentKey,
            rotationState: .failed(when: .packetTunnelCooldownInterval, nextKey: nextKey)
        )

        startDeviceCheck(
            remoteService: remoteService,
            deviceStateAccessor: deviceStateAccessor,
            rotateImmediatelyOnKeyMismatch: true
        ) { result in
            let deviceCheck = result.value

            XCTAssertNotNil(deviceCheck)
            XCTAssertTrue(deviceCheck?.keyRotationStatus.isSucceeded ?? false)
            XCTAssertEqual(try? deviceStateAccessor.read().deviceData?.wgKeyData.privateKey, nextKey)

            expect.fulfill()
        }

        await fulfillment(of: [expect], timeout: .UnitTest.timeout)
    }

    func testShouldRespectCooldownWhenAttemptingToRotateImmediately() async {
        let expect = expectation(description: "Wait for operation to complete")

        let currentKey = WireGuard.PrivateKey()
        let nextKey = WireGuard.PrivateKey()

        let remoteService = MockRemoteService()
        let deviceStateAccessor = MockDeviceStateAccessor.mockLoggedIn(
            currentKey: currentKey,
            rotationState: .failed(when: .zero, nextKey: nextKey)
        )

        startDeviceCheck(
            remoteService: remoteService,
            deviceStateAccessor: deviceStateAccessor,
            rotateImmediatelyOnKeyMismatch: true
        ) { result in
            let deviceCheck = result.value

            XCTAssertNotNil(deviceCheck)
            XCTAssertEqual(deviceCheck?.keyRotationStatus, KeyRotationStatus.noAction)
            XCTAssertEqual(try? deviceStateAccessor.read().deviceData?.wgKeyData.privateKey, currentKey)

            expect.fulfill()
        }

        await fulfillment(of: [expect], timeout: .UnitTest.timeout)
    }

    func testShouldNotRotateDeviceKeyWhenServerKeyIsIdentical() async {
        let expect = expectation(description: "Wait for operation to complete")

        let currentKey = WireGuard.PrivateKey()
        let remoteService = MockRemoteService(initialKey: currentKey.publicKey)
        let deviceStateAccessor = MockDeviceStateAccessor.mockLoggedIn(
            currentKey: currentKey,
            rotationState: .succeeded
        )

        startDeviceCheck(remoteService: remoteService, deviceStateAccessor: deviceStateAccessor) { result in
            let deviceCheck = result.value

            XCTAssertNotNil(deviceCheck)
            XCTAssertEqual(deviceCheck?.keyRotationStatus, KeyRotationStatus.noAction)
            XCTAssertEqual(try? deviceStateAccessor.read().deviceData?.wgKeyData.privateKey, currentKey)

            expect.fulfill()
        }

        await fulfillment(of: [expect], timeout: .UnitTest.timeout)
    }

    func testShouldNotRotateKeyBeforeRetryIntervalPassed() async {
        let expect = expectation(description: "Wait for operation to complete")

        let currentKey = WireGuard.PrivateKey()
        let nextKey = WireGuard.PrivateKey()

        let remoteService = MockRemoteService(initialKey: currentKey.publicKey)
        let deviceStateAccessor = MockDeviceStateAccessor.mockLoggedIn(
            currentKey: currentKey,
            rotationState: .failed(when: .closeToRetryInterval, nextKey: nextKey)
        )

        startDeviceCheck(remoteService: remoteService, deviceStateAccessor: deviceStateAccessor) { result in
            let deviceCheck = result.value

            XCTAssertNotNil(deviceCheck)
            XCTAssertEqual(deviceCheck?.keyRotationStatus, KeyRotationStatus.noAction)
            XCTAssertEqual(try? deviceStateAccessor.read().deviceData?.wgKeyData.privateKey, currentKey)

            expect.fulfill()
        }

        await fulfillment(of: [expect], timeout: .UnitTest.timeout)
    }

    func testShouldRotateKeyOnceInTwentyFourHours() async {
        let expect = expectation(description: "Wait for operation to complete")

        let currentKey = WireGuard.PrivateKey()
        let nextKey = WireGuard.PrivateKey()

        let remoteService = MockRemoteService()
        let deviceStateAccessor = MockDeviceStateAccessor.mockLoggedIn(
            currentKey: currentKey,
            rotationState: .failed(when: .retryInterval, nextKey: nextKey)
        )

        startDeviceCheck(remoteService: remoteService, deviceStateAccessor: deviceStateAccessor) { result in
            let deviceCheck = result.value

            XCTAssertNotNil(deviceCheck)
            XCTAssertTrue(deviceCheck?.keyRotationStatus.isSucceeded ?? false)
            XCTAssertEqual(try? deviceStateAccessor.read().deviceData?.wgKeyData.privateKey, nextKey)

            expect.fulfill()
        }

        await fulfillment(of: [expect], timeout: .UnitTest.timeout)
    }

    func testShouldReportFailedKeyRotationAttempt() async {
        let expect = expectation(description: "Wait for operation to complete")

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

        startDeviceCheck(remoteService: remoteService, deviceStateAccessor: deviceStateAccessor) { result in
            let deviceCheck = result.value

            XCTAssertNotNil(deviceCheck)
            XCTAssertTrue(deviceCheck?.keyRotationStatus.isAttempted ?? false)
            XCTAssertEqual(try? deviceStateAccessor.read().deviceData?.wgKeyData.privateKey, currentKey)

            expect.fulfill()
        }

        await fulfillment(of: [expect], timeout: .UnitTest.timeout)
    }

    func testShouldFailOnKeyRotationRace() async {
        let expect = expectation(description: "Wait for operation to complete")

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

        startDeviceCheck(remoteService: remoteService, deviceStateAccessor: deviceStateAccessor) { result in
            let deviceCheck = result.value

            XCTAssertNotNil(deviceCheck)
            XCTAssertTrue(deviceCheck?.keyRotationStatus.isAttempted ?? false)
            XCTAssertEqual(try? deviceStateAccessor.read().deviceData?.wgKeyData.privateKey, currentKey)

            expect.fulfill()
        }

        await fulfillment(of: [expect], timeout: .UnitTest.timeout)
    }

    private func startDeviceCheck(
        remoteService: DeviceCheckRemoteServiceProtocol,
        deviceStateAccessor: DeviceStateAccessorProtocol,
        rotateImmediatelyOnKeyMismatch: Bool = false,
        completion: @escaping (Result<DeviceCheck, Error>) -> Void
    ) {
        let operation = DeviceCheckOperation(
            dispatchQueue: dispatchQueue,
            remoteSevice: remoteService,
            deviceStateAccessor: deviceStateAccessor,
            rotateImmediatelyOnKeyMismatch: rotateImmediatelyOnKeyMismatch,
            completionHandler: completion
        )

        operationQueue.addOperation(operation)
    }
}
