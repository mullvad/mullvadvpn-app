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

/// What the actor should do after a device check.
public enum DeviceCheckOutcome: Sendable, Equatable {
    /// Account and device are in good standing, or the check could not be completed.
    case noAction

    /// A key rotation was attempted at the given date. The device state on disk has changed.
    case keyRotation(Date)

    /// The tunnel cannot work until the user intervenes.
    case blocked(BlockedStateReason)
}

/// Verifies account and device standing with the API and rotates the device key on mismatch.
public protocol DeviceCheckerProtocol: Sendable {
    /// `rotateKeyOnMismatch` forces rotation on key mismatch, subject to a short cooldown.
    /// Otherwise rotation respects the regular retry interval.
    func checkDevice(rotateKeyOnMismatch: Bool) async -> DeviceCheckOutcome
}
