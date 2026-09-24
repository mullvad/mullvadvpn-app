// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadSettings
import MullvadTypes
import os

@testable import MullvadMockData

/// Mock implementation of a remote service used by device checks to reach the API.
final class MockRemoteService: DeviceCheckRemoteServiceProtocol, Sendable {
    typealias AccountDataHandler = @Sendable (_ accountNumber: String) throws -> Account
    typealias DeviceDataHandler = @Sendable (_ accountNumber: String, _ deviceIdentifier: String) throws -> Device
    typealias RotateDeviceKeyHandler =
        @Sendable (
            _ accountNumber: String,
            _ deviceIdentifier: String,
            _ publicKey: WireGuard.PublicKey
        ) throws -> Void

    private let getAccountDataHandler: AccountDataHandler?
    private let getDeviceDataHandler: DeviceDataHandler?
    private let rotateDeviceKeyHandler: RotateDeviceKeyHandler?

    private let lockedKey: OSAllocatedUnfairLock<WireGuard.PublicKey>

    private var currentKey: WireGuard.PublicKey {
        get { lockedKey.withLock { $0 } }
        set { lockedKey.withLock { $0 = newValue } }
    }

    init(
        initialKey: WireGuard.PublicKey = WireGuard.PrivateKey().publicKey,
        getAccount: AccountDataHandler? = nil,
        getDevice: DeviceDataHandler? = nil,
        rotateDeviceKey: RotateDeviceKeyHandler? = nil
    ) {
        lockedKey = OSAllocatedUnfairLock(initialState: initialKey)
        getAccountDataHandler = getAccount
        getDeviceDataHandler = getDevice
        rotateDeviceKeyHandler = rotateDeviceKey
    }

    func getAccountData(accountNumber: String) async -> Result<Account, Error> {
        accountResult(accountNumber: accountNumber)
    }

    func getDevice(
        accountNumber: String,
        identifier: String,
        completion: @escaping @Sendable (Result<Device, Error>) -> Void
    ) -> Cancellable {
        DispatchQueue.main.async { [self] in
            completion(deviceResult(accountNumber: accountNumber, identifier: identifier))
        }

        return AnyCancellable()
    }

    func getDevice(accountNumber: String, identifier: String) async -> Result<Device, Error> {
        deviceResult(accountNumber: accountNumber, identifier: identifier)
    }

    func rotateDeviceKey(
        accountNumber: String,
        identifier: String,
        publicKey: WireGuard.PublicKey,
        completion: @escaping @Sendable (Result<Device, Error>) -> Void
    ) -> Cancellable {
        DispatchQueue.main.async { [self] in
            completion(rotationResult(accountNumber: accountNumber, identifier: identifier, publicKey: publicKey))
        }
        return AnyCancellable()
    }

    func rotateDeviceKey(
        accountNumber: String,
        identifier: String,
        publicKey: WireGuard.PublicKey
    ) async -> Result<Device, Error> {
        rotationResult(accountNumber: accountNumber, identifier: identifier, publicKey: publicKey)
    }

    private func accountResult(accountNumber: String) -> Result<Account, Error> {
        Result {
            if let getAccountDataHandler {
                try getAccountDataHandler(accountNumber)
            } else {
                Account.mock()
            }
        }
    }

    private func deviceResult(accountNumber: String, identifier: String) -> Result<Device, Error> {
        Result {
            if let getDeviceDataHandler {
                try getDeviceDataHandler(accountNumber, identifier)
            } else {
                Device.mock(publicKey: currentKey)
            }
        }
    }

    private func rotationResult(
        accountNumber: String,
        identifier: String,
        publicKey: WireGuard.PublicKey
    ) -> Result<Device, Error> {
        Result {
            try rotateDeviceKeyHandler?(accountNumber, identifier, publicKey)

            currentKey = publicKey

            return Device.mock(publicKey: currentKey)
        }
    }
}

/// Mock implementation of device state accessor used by device checks to access the storage holding device state.
final class MockDeviceStateAccessor: DeviceStateAccessorProtocol, Sendable {
    private let state: OSAllocatedUnfairLock<DeviceState>

    init(initialState: DeviceState) {
        state = OSAllocatedUnfairLock(initialState: initialState)
    }

    func read() throws -> DeviceState {
        state.withLock { $0 }
    }

    func write(_ deviceState: DeviceState) throws {
        state.withLock { $0 = deviceState }
    }
}

/// Time interval since last key rotation used for mocking `StoredWgKeyData`.
enum TimeSinceLastKeyRotation {
    /// No time passed since last key rotation.
    case zero

    /// Equal to key rotation retry interval minus 1 second.
    case closeToRetryInterval

    /// Equal to key rotation retry interval.
    case retryInterval

    /// Equal to cooldown interval used for packet tunnel based rotation.
    case packetTunnelCooldownInterval

    /// Returns negative time offset that can be used to compute the date in the past that can be used to simulate last
    /// attempt date when simulating key rotation.
    var timeOffset: TimeInterval {
        switch self {
        case .zero:
            return .zero
        case .closeToRetryInterval:
            return -WgKeyRotation.retryInterval.timeInterval + 1
        case .retryInterval:
            return -WgKeyRotation.retryInterval.timeInterval
        case .packetTunnelCooldownInterval:
            return -WgKeyRotation.packetTunnelCooldownInterval.timeInterval
        }
    }
}

/// State of last key rotation used for mocking `StoredWgKeyData`.
enum LastKeyRotationState {
    case succeeded
    case failed(when: TimeSinceLastKeyRotation, nextKey: WireGuard.PrivateKey)
}

extension MockDeviceStateAccessor {
    static func mockLoggedIn(currentKey: WireGuard.PrivateKey, rotationState: LastKeyRotationState)
        -> MockDeviceStateAccessor
    {
        MockDeviceStateAccessor(
            initialState: .loggedIn(
                StoredAccountData.mock(),
                StoredDeviceData.mock(
                    wgKeyData: StoredWgKeyData.mock(currentKey: currentKey, rotationState: rotationState))
            ))
    }
}

extension StoredWgKeyData {
    static func mock(currentKey: WireGuard.PrivateKey, rotationState: LastKeyRotationState) -> StoredWgKeyData {
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

extension StoredAccountData {
    static func mock() -> StoredAccountData {
        StoredAccountData(
            identifier: "account-id",
            number: "account-number",
            expiry: .distantFuture
        )
    }
}

extension StoredDeviceData {
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

extension KeyRotationStatus {
    /// Returns `true` if key rotation status is `.attempted`.
    var isAttempted: Bool {
        if case .attempted = self {
            return true
        }
        return false
    }
}

extension AccountVerdict {
    /// Returns `true` if account verdict is `.expired`.
    var isExpired: Bool {
        if case .expired = self {
            return true
        }
        return false
    }
}
