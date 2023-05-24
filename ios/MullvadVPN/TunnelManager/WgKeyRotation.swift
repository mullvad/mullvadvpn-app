//
//  WgKeyRotation.swift
//  MullvadVPN
//
//  Created by pronebird on 24/05/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import class WireGuardKitTypes.PrivateKey
import class WireGuardKitTypes.PublicKey

/**
 A simple manager that implements manipulations related to marking the beginning and the completion of key rotation,
 private key creation and other tasks relevant to handling the state of key rotation.
 */
struct WgKeyRotation {
    private(set) var data: StoredWgKeyData

    /**
     Initialize object with key data.
     */
    init(data: StoredWgKeyData) {
        self.data = data
    }

    /**
     Begin key rotation attempt by marking last rotation attempt and creating next private key if needed.
     If the next private key was created during the preivous rotation attempt then continue using the same key.

     Returns the public key that should be pushed to the backend.
     */
    mutating func beginAttempt() -> PublicKey {
        // Mark the rotation attempt.
        data.lastRotationAttemptDate = Date()

        // Fetch the next private key we're attempting to rotate to.
        if let nextPrivateKey = data.nextPrivateKey {
            return nextPrivateKey.publicKey
        } else {
            // If not found then create a new one and store it.
            let newKey = PrivateKey()
            data.nextPrivateKey = newKey
            return newKey.publicKey
        }
    }

    /**
     Successfuly finish key rotation by swapping the current key with the next one, marking key creation date and
     removing the date off last rotation attempt which indicates that the last rotation had succedeed and no new
     rotation attempts were made.
     */
    mutating func setCompleted() {
        guard let nextKey = data.nextPrivateKey else { return }

        // Reset creation date so that next period key rotation could happen relative to this date.
        data.creationDate = Date()

        // Swap old key and new.
        data.privateKey = nextKey
        data.nextPrivateKey = nil

        // Unset the date of last rotation attempt to mark the end of key rotation sequence.
        data.lastRotationAttemptDate = nil
    }

    /**
     Returns the date of next key rotation, as it normally occurs in the app process using the following rules:

     1. Returns the date relative to key creation date + 14 days, if last rotation attempt was successful.
     2. Returns the date relative to last rotation attempt date + 24 hours, if last rotation attempt was unsuccessful.

     If the date produced is in the past then `Date()` is returned instead.
     */
    func getNextRotationDate() -> Date {
        let rotationInterval: TimeInterval = 60 * 60 * 24 * 14
        let retryInterval: TimeInterval = 60 * 60 * 24

        let nextRotationDate = data.lastRotationAttemptDate?.addingTimeInterval(retryInterval)
            ?? data.creationDate.addingTimeInterval(rotationInterval)

        return max(nextRotationDate, Date())
    }

    /**
     Returns `true` if packet tunnel should perform key rotation.

     During the startup packet tunnel rotates the key immediately if it detected that the key stored on server does not
     match the key stored on device. In that case it passes `shouldRotateImmediately = true`.

     To dampen the effect of packet tunnel entering into a restart cycle and going on a key rotation rampage,
     this function adds a cooldown interval to prevent it from pushing keys too often.
     */
    func shouldPacketTunnelRotateTheKey(shouldRotateImmediately: Bool) -> Bool {
        guard let lastRotationAttemptDate = data.lastRotationAttemptDate else { return true }

        let now = Date()

        let retryInterval: TimeInterval = 60 * 60 * 24
        let cooldownInterval: TimeInterval = 15

        // Add cooldown interval when requested to rotate the key immediately.
        if shouldRotateImmediately, lastRotationAttemptDate.distance(to: now) > cooldownInterval {
            return true
        }

        let nextRotationAttempt = max(now, lastRotationAttemptDate.addingTimeInterval(retryInterval))
        if nextRotationAttempt <= now {
            return true
        }

        return false
    }
}
