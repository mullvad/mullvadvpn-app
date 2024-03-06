//
//  DeviceCheckOperationTests.swift
//  MullvadVPNTests
//
//  Created by pronebird on 30/05/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadSettings
import MullvadTypes
@testable import MullvadVPN
import Operations
import PacketTunnelCore
import WireGuardKitTypes
import XCTest

class DeviceCheckOperationTests: XCTestCase {
    private let operationQueue = AsyncOperationQueue()
    private let dispatchQueue = DispatchQueue(label: "TestQueue")

    func testShouldReportExpiredAccount() {
        let expect = expectation(description: "Wait for operation to complete")

        let currentKey = PrivateKey()
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

        waitForExpectations(timeout: 1)
    }

    func testShouldNotRotateKeyForInvalidAccount() {
        let expect = expectation(description: "Wait for operation to complete")

        let currentKey = PrivateKey()
        let nextKey = PrivateKey()

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

        waitForExpectations(timeout: 1)
    }

    func testShouldNotRotateKeyForRevokedDevice() {
        let expect = expectation(description: "Wait for operation to complete")

        let currentKey = PrivateKey()
        let nextKey = PrivateKey()

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

        waitForExpectations(timeout: 1)
    }

    func testShouldRotateKeyOnMismatchImmediately() {
        let expect = expectation(description: "Wait for operation to complete")

        let currentKey = PrivateKey()
        let nextKey = PrivateKey()

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

        waitForExpectations(timeout: 1)
    }

    func testShouldRespectCooldownWhenAttemptingToRotateImmediately() {
        let expect = expectation(description: "Wait for operation to complete")

        let currentKey = PrivateKey()
        let nextKey = PrivateKey()

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

        waitForExpectations(timeout: 1)
    }

    func testShouldNotRotateDeviceKeyWhenServerKeyIsIdentical() {
        let expect = expectation(description: "Wait for operation to complete")

        let currentKey = PrivateKey()
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

        waitForExpectations(timeout: 1)
    }

    func testShouldNotRotateKeyBeforeRetryIntervalPassed() {
        let expect = expectation(description: "Wait for operation to complete")

        let currentKey = PrivateKey()
        let nextKey = PrivateKey()

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

        waitForExpectations(timeout: 1)
    }

    func testShouldRotateKeyOnceInTwentyFourHours() {
        let expect = expectation(description: "Wait for operation to complete")

        let currentKey = PrivateKey()
        let nextKey = PrivateKey()

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

        waitForExpectations(timeout: 1)
    }

    func testShouldReportFailedKeyRotataionAttempt() {
        let expect = expectation(description: "Wait for operation to complete")

        let currentKey = PrivateKey()
        let nextKey = PrivateKey()

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

        waitForExpectations(timeout: 1)
    }

    func testShouldFailOnKeyRotationRace() {
        let expect = expectation(description: "Wait for operation to complete")

        let currentKey = PrivateKey()
        let nextKey = PrivateKey()

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

        waitForExpectations(timeout: 1)
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

/// Mock implementation of a remote service used by `DeviceCheckOperation` to reach the API.
private class MockRemoteService: DeviceCheckRemoteServiceProtocol {
    typealias AccountDataHandler = (_ accountNumber: String) throws -> Account
    typealias DeviceDataHandler = (_ accountNumber: String, _ deviceIdentifier: String) throws -> Device
    typealias RotateDeviceKeyHandler = (
        _ accountNumber: String,
        _ deviceIdentifier: String,
        _ publicKey: PublicKey
    ) throws -> Void

    private let getAccountDataHandler: AccountDataHandler?
    private let getDeviceDataHandler: DeviceDataHandler?
    private let rotateDeviceKeyHandler: RotateDeviceKeyHandler?

    private var currentKey: PublicKey

    init(
        initialKey: PublicKey = PrivateKey().publicKey,
        getAccount: AccountDataHandler? = nil,
        getDevice: DeviceDataHandler? = nil,
        rotateDeviceKey: RotateDeviceKeyHandler? = nil
    ) {
        currentKey = initialKey
        getAccountDataHandler = getAccount
        getDeviceDataHandler = getDevice
        rotateDeviceKeyHandler = rotateDeviceKey
    }

    func getAccountData(
        accountNumber: String,
        completion: @escaping (Result<Account, Error>) -> Void
    ) -> Cancellable {
        DispatchQueue.main.async { [self] in
            let result: Result<Account, Error> = Result {
                if let getAccountDataHandler {
                    return try getAccountDataHandler(accountNumber)
                } else {
                    return Account.mock()
                }
            }
            completion(result)
        }
        return AnyCancellable()
    }

    func getDevice(
        accountNumber: String,
        identifier: String,
        completion: @escaping (Result<Device, Error>) -> Void
    ) -> Cancellable {
        DispatchQueue.main.async { [self] in
            let result: Result<Device, Error> = Result {
                if let getDeviceDataHandler {
                    return try getDeviceDataHandler(accountNumber, identifier)
                } else {
                    return Device.mock(publicKey: currentKey)
                }
            }

            completion(result)
        }

        return AnyCancellable()
    }

    func rotateDeviceKey(
        accountNumber: String,
        identifier: String,
        publicKey: PublicKey,
        completion: @escaping (Result<Device, Error>) -> Void
    ) -> Cancellable {
        DispatchQueue.main.async { [self] in
            let result: Result<Device, Error> = Result {
                try rotateDeviceKeyHandler?(accountNumber, identifier, publicKey)

                currentKey = publicKey

                return Device.mock(publicKey: currentKey)
            }

            completion(result)
        }
        return AnyCancellable()
    }
}

/// Mock implementation of device state accessor used by `CheckDeviceOperation` to access the storage holding device
/// state.
private class MockDeviceStateAccessor: DeviceStateAccessorProtocol {
    private var state: DeviceState
    private let stateLock = NSLock()

    init(initialState: DeviceState) {
        state = initialState
    }

    func read() throws -> DeviceState {
        stateLock.lock()
        defer { stateLock.unlock() }
        return state
    }

    func write(_ deviceState: DeviceState) throws {
        stateLock.lock()
        defer { stateLock.unlock() }
        state = deviceState
    }
}

/// Time interval since last key rotation used for mocking `StoredWgKeyData`.
private enum TimeSinceLastKeyRotation {
    /// No time passed since last key rotation.
    case zero

    /// Equal to key rotation retry interval.
    case retryInterval

    /// Equal to key rotation retry interval minus 1 second.
    case closeToRetryInterval

    /// Equal to cooldown interval used for packet tunnel based rotation.
    case packetTunnelCooldownInterval

    /// Returns negative time offset that can be used to compute the date in the past that can be used to simulate last
    /// attempt date when simulating key rotation.
    var timeOffset: TimeInterval {
        switch self {
        case .zero:
            return .zero
        case .retryInterval:
            return -WgKeyRotation.retryInterval.timeInterval
        case .closeToRetryInterval:
            return -WgKeyRotation.retryInterval.timeInterval + 1
        case .packetTunnelCooldownInterval:
            return -WgKeyRotation.packetTunnelCooldownInterval.timeInterval
        }
    }
}

/// State of last key rotation used for mocking `StoredWgKeyData`.
private enum LastKeyRotationState {
    case succeeded
    case failed(when: TimeSinceLastKeyRotation, nextKey: PrivateKey)
}

extension MockDeviceStateAccessor {
    static func mockLoggedIn(currentKey: PrivateKey, rotationState: LastKeyRotationState) -> MockDeviceStateAccessor {
        MockDeviceStateAccessor(initialState: .loggedIn(
            StoredAccountData.mock(),
            StoredDeviceData.mock(wgKeyData: StoredWgKeyData.mock(currentKey: currentKey, rotationState: rotationState))
        ))
    }
}

private extension StoredWgKeyData {
    static func mock(currentKey: PrivateKey, rotationState: LastKeyRotationState) -> StoredWgKeyData {
        var keyData = StoredWgKeyData(creationDate: Date(), privateKey: currentKey)
        keyData.apply(rotationState)
        return keyData
    }

    private mutating func apply(_ rotationState: LastKeyRotationState) {
        switch rotationState {
        case .succeeded:
            lastRotationAttemptDate = nil
            nextPrivateKey = nil

        case let .failed(recency, nextKey):
            let attemptDate = creationDate.addingTimeInterval(recency.timeOffset)

            creationDate = min(creationDate, attemptDate)
            lastRotationAttemptDate = attemptDate
            nextPrivateKey = nextKey
        }
    }
}

private extension StoredAccountData {
    static func mock() -> StoredAccountData {
        StoredAccountData(
            identifier: "account-id",
            number: "account-number",
            expiry: .distantFuture
        )
    }
}

private extension StoredDeviceData {
    static func mock(wgKeyData: StoredWgKeyData) -> StoredDeviceData {
        StoredDeviceData(
            creationDate: Date(),
            identifier: "device-id",
            name: "device-name",
            hijackDNS: false,
            ipv4Address: IPAddressRange(from: "127.0.0.1/32")!,
            ipv6Address: IPAddressRange(from: "::ff/64")!,
            wgKeyData: wgKeyData
        )
    }
}

private extension KeyRotationStatus {
    /// Returns `true` if key rotation status is `.attempted`.
    var isAttempted: Bool {
        if case .attempted = self {
            return true
        }
        return false
    }
}

private extension AccountVerdict {
    /// Returns `true` if account verdict is `.expired`.
    var isExpired: Bool {
        if case .expired = self {
            return true
        }
        return false
    }

    // swiftlint:disable:next file_length
}
