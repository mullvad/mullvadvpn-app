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
import MullvadLogging
import MullvadSettings
import MullvadTypes
import PacketTunnelCore

/// Async counterpart of `DeviceCheckOperation`: fetches account and device data concurrently and rotates the
/// device key on mismatch. Holds no state between checks.
final class AsyncDeviceChecker: Sendable {
    private let logger = Logger(label: "AsyncDeviceChecker")
    private let remoteService: DeviceCheckRemoteServiceProtocol
    private let deviceStateAccessor: DeviceStateAccessorProtocol

    init(remoteService: DeviceCheckRemoteServiceProtocol, deviceStateAccessor: DeviceStateAccessorProtocol) {
        self.remoteService = remoteService
        self.deviceStateAccessor = deviceStateAccessor
    }

    /// With `rotateImmediatelyOnKeyMismatch` the key is rotated on mismatch unless the last attempt was less than
    /// `WgKeyRotation.packetTunnelCooldownInterval` ago. Otherwise rotation waits for `WgKeyRotation.retryInterval`.
    func check(rotateImmediatelyOnKeyMismatch: Bool) async -> Result<DeviceCheck, Error> {
        do {
            let (accountData, deviceData) = try loggedInState()

            async let accountTask = remoteService.getAccountData(accountNumber: accountData.number)
            async let deviceTask = remoteService.getDevice(
                accountNumber: accountData.number,
                identifier: deviceData.identifier
            )
            let (accountResult, deviceResult) = await (accountTask, deviceTask)

            var deviceCheck = DeviceCheck(
                accountVerdict: try AccountVerdict(accountResult: accountResult),
                deviceVerdict: try DeviceVerdict(deviceResult: deviceResult, deviceState: deviceStateAccessor.read()),
                keyRotationStatus: .noAction
            )

            // Do not rotate the key if account is invalid even if the API successfully returns a device.
            if deviceCheck.accountVerdict != .invalid, deviceCheck.deviceVerdict == .keyMismatch {
                let rotationStatus = try await rotateKeyIfNeeded(rotateImmediately: rotateImmediatelyOnKeyMismatch)
                deviceCheck.keyRotationStatus = rotationStatus
                if rotationStatus.isSucceeded {
                    deviceCheck.deviceVerdict = .active
                }
            }

            return .success(deviceCheck)
        } catch {
            return .failure(error)
        }
    }

    private func loggedInState() throws -> (StoredAccountData, StoredDeviceData) {
        guard case let .loggedIn(accountData, deviceData) = try deviceStateAccessor.read() else {
            throw DeviceCheckError.invalidDeviceState
        }
        return (accountData, deviceData)
    }

    private func rotateKeyIfNeeded(rotateImmediately: Bool) async throws -> KeyRotationStatus {
        let (accountData, deviceData) = try loggedInState()

        var keyRotation = WgKeyRotation(data: deviceData)
        guard keyRotation.shouldRotateFromPacketTunnel(rotateImmediately: rotateImmediately) else {
            return .noAction
        }

        let publicKey = keyRotation.beginAttempt()

        // Persist the attempt date and the pending key before the request, so that a restart loop honours the
        // cooldown and a retry pushes the same key again.
        try deviceStateAccessor.write(.loggedIn(accountData, keyRotation.data))

        logger.debug("Rotate private key from packet tunnel.")

        let rotationResult = await remoteService.rotateDeviceKey(
            accountNumber: accountData.number,
            identifier: deviceData.identifier,
            publicKey: publicKey
        )

        do {
            try completeKeyRotation(try rotationResult.get())
            return .succeeded(Date())
        } catch is CancellationError {
            throw CancellationError()
        } catch {
            logger.error(error: error, message: "Failed to rotate device key.")
            return .attempted(Date())
        }
    }

    /// Swaps in the new private key. Fails with `keyRotationRace` if the pending key was cleared by the app while
    /// the request was in flight.
    private func completeKeyRotation(_ device: Device) throws {
        let (accountData, deviceData) = try loggedInState()

        var keyRotation = WgKeyRotation(data: deviceData)
        guard keyRotation.setCompleted(with: device) else {
            throw DeviceCheckError.keyRotationRace
        }

        try deviceStateAccessor.write(.loggedIn(accountData, keyRotation.data))
        logger.debug("Successfully rotated device key.")
    }
}

extension AsyncDeviceChecker: DeviceCheckerProtocol {
    func checkDevice(rotateKeyOnMismatch: Bool) async -> DeviceCheckOutcome {
        switch await check(rotateImmediatelyOnKeyMismatch: rotateKeyOnMismatch) {
        case let .failure(error):
            if error is DeviceCheckError {
                logger.error("\(error.description) Forcing a log out.")
                return .blocked(.deviceLoggedOut)
            }
            logger.error("Device check encountered a network error: \(error.description)")
            return .noAction

        case let .success(deviceCheck):
            if let reason = deviceCheck.blockedStateReason {
                return .blocked(reason)
            }
            switch deviceCheck.keyRotationStatus {
            case let .attempted(date), let .succeeded(date):
                return .keyRotation(date)
            case .noAction:
                return .noAction
            }
        }
    }
}
