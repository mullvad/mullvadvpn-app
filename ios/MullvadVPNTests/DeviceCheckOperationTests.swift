//
//  DeviceCheckOperationTests.swift
//  MullvadVPNTests
//
//  Created by pronebird on 30/05/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadTypes
import Operations
import WireGuardKitTypes
import XCTest

class DeviceCheckOperationTests: XCTestCase {
    let operationQueue = AsyncOperationQueue()
    let dispatchQueue = DispatchQueue(label: "TestQueue")

    func testShouldReportExpiredAccount() {
        let expect = expectation(description: "Wait for operation to complete")

        let currentKey = PrivateKey()
        let remoteService = MockRemoteService(
            initialKey: currentKey,
            getAccount: { accountNumber in
                return Account.mock(expiry: .distantPast)
            }
        )
        let deviceStateAccessor = MockDeviceStateAccessor(
            initialState: .loggedIn(
                StoredAccountData.mock(),
                StoredDeviceData.mock(wgKeyData: StoredWgKeyData(creationDate: Date(), privateKey: currentKey))
            )
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
        let remoteService = MockRemoteService(
            initialKey: currentKey,
            getAccount: { accountNumber in
                throw REST.Error.unhandledResponse(404, REST.ServerErrorResponse(code: .invalidAccount))
            }
        )
        let deviceStateAccessor = MockDeviceStateAccessor(
            initialState: .loggedIn(
                StoredAccountData.mock(),
                StoredDeviceData.mock(
                    wgKeyData: StoredWgKeyData(
                        creationDate: Date(),
                        lastRotationAttemptDate: Date().addingTimeInterval(-WgKeyRotation.retryInterval),
                        privateKey: currentKey,
                        nextPrivateKey: PrivateKey()
                    )
                )
            )
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
        let remoteService = MockRemoteService(
            initialKey: currentKey,
            getDevice: { accountNumber, deviceIdentifier in
                throw REST.Error.unhandledResponse(404, REST.ServerErrorResponse(code: .deviceNotFound))
            }
        )
        let deviceStateAccessor = MockDeviceStateAccessor(
            initialState: .loggedIn(
                StoredAccountData.mock(),
                StoredDeviceData.mock(
                    wgKeyData: StoredWgKeyData(
                        creationDate: Date(),
                        lastRotationAttemptDate: Date().addingTimeInterval(-WgKeyRotation.retryInterval),
                        privateKey: currentKey,
                        nextPrivateKey: PrivateKey()
                    )
                )
            )
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

        let nextKey = PrivateKey()
        let remoteService = MockRemoteService()
        let deviceStateAccessor = MockDeviceStateAccessor(
            initialState: .loggedIn(
                StoredAccountData.mock(),
                StoredDeviceData.mock(
                    wgKeyData: StoredWgKeyData(
                        creationDate: Date(),
                        lastRotationAttemptDate: Date().addingTimeInterval(-WgKeyRotation.packetTunnelCooldownInterval),
                        privateKey: PrivateKey(),
                        nextPrivateKey: nextKey
                    )
                )
            )
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
        let remoteService = MockRemoteService()
        let deviceStateAccessor = MockDeviceStateAccessor(
            initialState: .loggedIn(
                StoredAccountData.mock(),
                StoredDeviceData.mock(
                    wgKeyData: StoredWgKeyData(
                        creationDate: Date(),
                        lastRotationAttemptDate: Date(),
                        privateKey: currentKey,
                        nextPrivateKey: PrivateKey()
                    )
                )
            )
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

        let deviceKey = PrivateKey()
        let remoteService = MockRemoteService(initialKey: deviceKey)
        let deviceStateAccessor = MockDeviceStateAccessor(
            initialState: .loggedIn(
                StoredAccountData.mock(),
                StoredDeviceData.mock(
                    wgKeyData: StoredWgKeyData(
                        creationDate: Date(),
                        lastRotationAttemptDate: nil,
                        privateKey: deviceKey,
                        nextPrivateKey: nil
                    )
                )
            )
        )

        startDeviceCheck(remoteService: remoteService, deviceStateAccessor: deviceStateAccessor) { result in
            let deviceCheck = result.value

            XCTAssertNotNil(deviceCheck)
            XCTAssertEqual(deviceCheck?.keyRotationStatus, KeyRotationStatus.noAction)
            XCTAssertEqual(try? deviceStateAccessor.read().deviceData?.wgKeyData.privateKey, deviceKey)

            expect.fulfill()
        }

        waitForExpectations(timeout: 1)
    }

    func testShouldNotRotateKeyBeforeTwentyFourHoursHavePassed() {
        let expect = expectation(description: "Wait for operation to complete")

        let currentKey = PrivateKey()
        let remoteService = MockRemoteService()
        let deviceStateAccessor = MockDeviceStateAccessor(
            initialState: .loggedIn(
                StoredAccountData.mock(),
                StoredDeviceData.mock(
                    wgKeyData: StoredWgKeyData(
                        creationDate: Date(),
                        lastRotationAttemptDate: Date().addingTimeInterval(-WgKeyRotation.retryInterval + 1),
                        privateKey: currentKey,
                        nextPrivateKey: PrivateKey()
                    )
                )
            )
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

        let nextKey = PrivateKey()
        let remoteService = MockRemoteService()
        let deviceStateAccessor = MockDeviceStateAccessor(
            initialState: .loggedIn(
                StoredAccountData.mock(),
                StoredDeviceData.mock(
                    wgKeyData: StoredWgKeyData(
                        creationDate: Date(),
                        lastRotationAttemptDate: Date().addingTimeInterval(-WgKeyRotation.retryInterval),
                        privateKey: PrivateKey(),
                        nextPrivateKey: nextKey
                    )
                )
            )
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
        let remoteService = MockRemoteService(
            rotateDeviceKey: { accountNumber, identifier, publicKey in
                throw URLError(.badURL)
            }
        )

        let deviceStateAccessor = MockDeviceStateAccessor(
            initialState: .loggedIn(
                StoredAccountData.mock(),
                StoredDeviceData.mock(
                    wgKeyData: StoredWgKeyData(
                        creationDate: Date(),
                        lastRotationAttemptDate: Date().addingTimeInterval(-WgKeyRotation.retryInterval),
                        privateKey: currentKey,
                        nextPrivateKey: PrivateKey()
                    )
                )
            )
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
        let deviceStateAccessor = MockDeviceStateAccessor(
            initialState: .loggedIn(
                StoredAccountData.mock(),
                StoredDeviceData.mock(
                    wgKeyData: StoredWgKeyData(
                        creationDate: Date(),
                        lastRotationAttemptDate: Date().addingTimeInterval(-WgKeyRotation.retryInterval),
                        privateKey: currentKey,
                        nextPrivateKey: PrivateKey()
                    )
                )
            )
        )

        let remoteService = MockRemoteService(
            rotateDeviceKey: { accountNumber, identifier, publicKey in
                // Overwrite device state before returning the result from key rotation to simulate the race condition
                // in the underlying storage.
                try deviceStateAccessor.write(
                    .loggedIn(
                        StoredAccountData.mock(),
                        StoredDeviceData.mock(wgKeyData: StoredWgKeyData(creationDate: Date(), privateKey: currentKey))
                    )
                )

                return PrivateKey()
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

/// Mock implemntation of a remote service used by `DeviceCheckOperation` to reach the API.
private class MockRemoteService: DeviceCheckRemoteServiceProtocol {
    typealias AccountDataHandler = (_ accountNumber: String) throws -> Account
    typealias DeviceDataHandler = (_ accountNumber: String, _ deviceIdentifier: String) throws -> Device
    typealias RotateDeviceKeyHandler = (_ accountNumber: String, _ identifier: String, _ publicKey: PublicKey)
        throws -> PrivateKey

    private let getAccountDataHandler: AccountDataHandler?
    private let getDeviceDataHandler: DeviceDataHandler?
    private let rotateDeviceKeyHandler: RotateDeviceKeyHandler?

    private var currentKey: PrivateKey

    init(
        initialKey: PrivateKey = PrivateKey(),
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
                    return Device.mock(privateKey: currentKey)
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
                if let rotateDeviceKeyHandler {
                    currentKey = try rotateDeviceKeyHandler(accountNumber, identifier, publicKey)
                } else {
                    currentKey = PrivateKey()
                }

                return Device.mock(privateKey: currentKey)
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

private extension StoredAccountData {
    static func mock() -> StoredAccountData {
        return StoredAccountData(
            identifier: "account-id",
            number: "account-number",
            expiry: .distantFuture
        )
    }
}

private extension StoredDeviceData {
    static func mock(wgKeyData: StoredWgKeyData) -> StoredDeviceData {
        return StoredDeviceData(
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

private extension Device {
    static func mock(privateKey: PrivateKey) -> Device {
        return Device(
            id: "device-id",
            name: "device-name",
            pubkey: privateKey.publicKey,
            hijackDNS: false,
            created: Date(),
            ipv4Address: IPAddressRange(from: "127.0.0.1/32")!,
            ipv6Address: IPAddressRange(from: "::ff/64")!,
            ports: []
        )
    }
}

private extension Account {
    static func mock(expiry: Date = .distantFuture) -> Account {
        return Account(
            id: "account-id",
            expiry: expiry,
            maxPorts: 5,
            canAddPorts: true,
            maxDevices: 5,
            canAddDevices: true
        )
    }
}

private extension KeyRotationStatus {
    var isAttempted: Bool {
        if case .attempted = self {
            return true
        }
        return false
    }
}

private extension AccountVerdict {
    var isExpired: Bool {
        if case .expired = self {
            return true
        }
        return false
    }
}
