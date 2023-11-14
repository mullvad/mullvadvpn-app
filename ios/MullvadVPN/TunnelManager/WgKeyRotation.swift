//
//  WgKeyRotation.swift
//  MullvadVPN
//
//  Created by pronebird on 24/05/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import MullvadTypes
import WireGuardKitTypes

/**
 Implements manipulations related to marking the beginning and the completion of key rotation, private key creation and other tasks relevant to handling the state of
 key rotation.
 */
struct WgKeyRotation {
    /// Private key rotation interval counted from the time when the key was successfully pushed
    /// to the backend.
    public static let rotationInterval: Duration = .days(14)

    /// Private key rotation retry interval counted from the time when the last rotation
    /// attempt took place.
    public static let retryInterval: Duration = .days(1)

    /// Cooldown interval used to prevent packet tunnel from forcefully pushing the key to our
    /// backend in the event of restart loop.
    public static let packetTunnelCooldownInterval: Duration = .seconds(15)

    /// Mutated device data value.
    private(set) var data: StoredDeviceData

    /// Initialize object with `StoredDeviceData` that the struct is going to manipulate.
    init(data: StoredDeviceData) {
        self.data = data
    }

    /**
     Begin key rotation attempt by marking last rotation attempt and creating next private key if needed.
     If the next private key was created during the preivous rotation attempt then continue using the same key.

     Returns the public key that should be pushed to the backend.
     */
    mutating func beginAttempt() -> PublicKey {
        // Mark the rotation attempt.
        data.wgKeyData.lastRotationAttemptDate = Date()

        // Fetch the next private key we're attempting to rotate to.
        if let nextPrivateKey = data.wgKeyData.nextPrivateKey {
            return nextPrivateKey.publicKey
        } else {
            // If not found then create a new one and store it.
            let newKey = PrivateKey()
            data.wgKeyData.nextPrivateKey = newKey
            return newKey.publicKey
        }
    }

    /**
     Successfuly finish key rotation by swapping the current key with the next one, marking key creation date and
     removing the date of last rotation attempt which indicates that the last rotation had succedeed and no new
     rotation attempts were made.

     Device related properties are refreshed from `Device` struct that the caller should have received from the API. This function does nothing if the next private
     key is unset.

     Returns `false` if next private key is unset. Otherwise `true`.
     */
    mutating func setCompleted(with updatedDevice: Device) -> Bool {
        guard let nextKey = data.wgKeyData.nextPrivateKey else { return false }

        // Update stored device data with properties from updated `Device` struct.
        data.update(from: updatedDevice)

        // Reset creation date so that next period key rotation could happen relative to this date.
        data.wgKeyData.creationDate = Date()

        // Swap old and new keys.
        data.wgKeyData.privateKey = nextKey
        data.wgKeyData.nextPrivateKey = nil

        // Unset the date of last rotation attempt to mark the end of key rotation sequence.
        data.wgKeyData.lastRotationAttemptDate = nil

        return true
    }

    /**
     Returns the date of next key rotation, as it normally occurs in the app process using the following rules:

     1. Returns the date relative to key creation date + 14 days, if last rotation attempt was successful.
     2. Returns the date relative to last rotation attempt date + 24 hours, if last rotation attempt was unsuccessful.

     If the date produced is in the past then `Date()` is returned instead.
     */
    var nextRotationDate: Date {
        let nextRotationDate = data.wgKeyData.lastRotationAttemptDate?
            .addingTimeInterval(Self.retryInterval.timeInterval)
            ?? data.wgKeyData.creationDate.addingTimeInterval(Self.rotationInterval.timeInterval)

        return max(nextRotationDate, Date())
    }

    /// Returns `true` if the app should rotate the private key.
    var shouldRotate: Bool {
        nextRotationDate <= Date()
    }

    /**
     Returns `true` if packet tunnel should perform key rotation.

     During the startup packet tunnel rotates the key immediately if it detected that the key stored on server does not
     match the key stored on device. In that case it passes `rotateImmediately = true`.

     To dampen the effect of packet tunnel entering into a restart cycle and going on a key rotation rampage,
     this function adds a 15 seconds cooldown interval to prevent it from pushing keys too often.

     After performing the initial key rotation on startup, packet tunnel will keep a 24 hour interval between the
     subsequent key rotation attempts.
     */
    func shouldRotateFromPacketTunnel(rotateImmediately: Bool) -> Bool {
        guard let lastRotationAttemptDate = data.wgKeyData.lastRotationAttemptDate else { return true }

        let now = Date()

        // Add cooldown interval when requested to rotate the key immediately.
        if rotateImmediately, lastRotationAttemptDate.distance(to: now) > Self.packetTunnelCooldownInterval {
            return true
        }

        let nextRotationAttempt = max(now, lastRotationAttemptDate.addingTimeInterval(Self.retryInterval.timeInterval))
        if nextRotationAttempt <= now {
            return true
        }

        return false
    }
}
