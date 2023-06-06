//
//  DeviceCheckOperationTests.swift
//  MullvadVPNTests
//
//  Created by pronebird on 30/05/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import Operations
import WireGuardKitTypes
import XCTest

class DeviceCheckOperationTests: XCTestCase {
    let operationQueue = AsyncOperationQueue()
    let dispatchQueue = DispatchQueue(label: "TestQueue")

    func testShouldRotateKeyOnMismatchImmediately() {
        let expect = expectation(description: "Wait for operation to complete")

        let nextKey = PrivateKey()
        let remoteService = MockRemoteService(initialKey: PrivateKey())
        let deviceStateAccessor = MockDeviceStateAccessor(
            initialState: .loggedIn(
                StoredAccountData.mock(),
                StoredDeviceData.mock(
                    wgKeyData: StoredWgKeyData(
                        creationDate: Date(),
                        lastRotationAttemptDate: Date().addingTimeInterval(-15),
                        privateKey: PrivateKey(),
                        nextPrivateKey: nextKey
                    )
                )
            )
        )

        startDeviceCheck(
            remoteService: remoteService,
            deviceStateAccessor: deviceStateAccessor,
            shouldImmediatelyRotateKeyOnMismatch: true
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
        let remoteService = MockRemoteService(initialKey: PrivateKey())
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
            shouldImmediatelyRotateKeyOnMismatch: true
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

        startDeviceCheck(
            remoteService: remoteService,
            deviceStateAccessor: deviceStateAccessor,
            shouldImmediatelyRotateKeyOnMismatch: false
        ) { result in
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
        let remoteService = MockRemoteService(initialKey: PrivateKey())
        let deviceStateAccessor = MockDeviceStateAccessor(
            initialState: .loggedIn(
                StoredAccountData.mock(),
                StoredDeviceData.mock(
                    wgKeyData: StoredWgKeyData(
                        creationDate: Date(),
                        lastRotationAttemptDate: Date().addingTimeInterval(-86399),
                        privateKey: currentKey,
                        nextPrivateKey: PrivateKey()
                    )
                )
            )
        )

        startDeviceCheck(
            remoteService: remoteService,
            deviceStateAccessor: deviceStateAccessor,
            shouldImmediatelyRotateKeyOnMismatch: false
        ) { result in
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
        let remoteService = MockRemoteService(initialKey: PrivateKey())
        let deviceStateAccessor = MockDeviceStateAccessor(
            initialState: .loggedIn(
                StoredAccountData.mock(),
                StoredDeviceData.mock(
                    wgKeyData: StoredWgKeyData(
                        creationDate: Date(),
                        lastRotationAttemptDate: Date().addingTimeInterval(-86400),
                        privateKey: PrivateKey(),
                        nextPrivateKey: nextKey
                    )
                )
            )
        )

        startDeviceCheck(
            remoteService: remoteService,
            deviceStateAccessor: deviceStateAccessor,
            shouldImmediatelyRotateKeyOnMismatch: false
        ) { result in
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
            initialKey: PrivateKey(),
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
                        lastRotationAttemptDate: Date().addingTimeInterval(-86400),
                        privateKey: currentKey,
                        nextPrivateKey: PrivateKey()
                    )
                )
            )
        )

        startDeviceCheck(
            remoteService: remoteService,
            deviceStateAccessor: deviceStateAccessor,
            shouldImmediatelyRotateKeyOnMismatch: false
        ) { result in
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
                        lastRotationAttemptDate: Date().addingTimeInterval(-86400),
                        privateKey: currentKey,
                        nextPrivateKey: PrivateKey()
                    )
                )
            )
        )

        let remoteService = MockRemoteService(
            initialKey: PrivateKey(),
            rotateDeviceKey: { accountNumber, identifier, publicKey in
                // Overwrite device state before returning the result from key rotation to simulate the race condition
                // in the underlying storage.
                try deviceStateAccessor.write(
                    .loggedIn(
                        StoredAccountData.mock(),
                        StoredDeviceData.mock(wgKeyData: StoredWgKeyData(creationDate: Date(), privateKey: currentKey))
                    )
                )

                let newKey = PrivateKey()
                return .init(newPrivateKey: newKey, device: Device.mock(privateKey: newKey))
            }
        )

        startDeviceCheck(
            remoteService: remoteService,
            deviceStateAccessor: deviceStateAccessor,
            shouldImmediatelyRotateKeyOnMismatch: false
        ) { result in
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
        shouldImmediatelyRotateKeyOnMismatch: Bool,
        completion: @escaping (Result<DeviceCheck, Error>) -> Void
    ) {
        let operation = DeviceCheckOperation(
            dispatchQueue: dispatchQueue,
            remoteSevice: remoteService,
            deviceStateAccessor: deviceStateAccessor,
            shouldImmediatelyRotateKeyOnMismatch: shouldImmediatelyRotateKeyOnMismatch,
            completionHandler: completion
        )

        operationQueue.addOperation(operation)
    }
}

/// Mock implemntation of a remote service used by `DeviceCheckOperation` to reach the API.
private class MockRemoteService: DeviceCheckRemoteServiceProtocol {
    typealias RotateDeviceKeyHandler = (_ accountNumber: String, _ identifier: String, _ publicKey: PublicKey)
        throws -> PrivateKey

    private let rotateDeviceKeyHandler: RotateDeviceKeyHandler?

    private var currentKey: PrivateKey

    init(
        initialKey: PrivateKey,
        rotateDeviceKey: RotateDeviceKeyHandler? = nil
    ) {
        currentKey = initialKey
        rotateDeviceKeyHandler = rotateDeviceKey
    }

    func getAccountData(
        accountNumber: String,
        completion: @escaping (Result<Account, Error>) -> Void
    ) -> Cancellable {
        DispatchQueue.main.async {
            completion(.success(Account.mock()))
        }
        return AnyCancellable()
    }

    func getDevice(
        accountNumber: String,
        identifier: String,
        completion: @escaping (Result<Device, Error>) -> Void
    ) -> Cancellable {
        DispatchQueue.main.async { [self] in
            completion(.success(Device.mock(privateKey: currentKey)))
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
            expiry: Date().addingTimeInterval(86400)
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
    static func mock() -> Account {
        return Account(
            id: "account-id",
            expiry: Date().addingTimeInterval(86400),
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
