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
import NetworkExtension

struct ProviderStopReasonWrapper: CustomStringConvertible {
    let reason: NEProviderStopReason

    public var description: String {
        switch reason {
        case .none:
            return "none"
        case .userInitiated:
            return "user initiated"
        case .providerFailed:
            return "provider failed"
        case .noNetworkAvailable:
            return "no network available"
        case .unrecoverableNetworkChange:
            return "unrecoverable network change"
        case .providerDisabled:
            return "provider disabled"
        case .authenticationCanceled:
            return "authentication cancelled"
        case .configurationFailed:
            return "configuration failed"
        case .idleTimeout:
            return "idle timeout"
        case .configurationDisabled:
            return "configuration disabled"
        case .configurationRemoved:
            return "configuration removed"
        case .superceded:
            return "superceded"
        case .userLogout:
            return "user logout"
        case .userSwitch:
            return "user switch"
        case .connectionFailed:
            return "connection failed"
        case .sleep:
            return "sleep"
        case .appUpdate:
            return "app update"
        case .internalError:
            return "internal error"
        @unknown default:
            return "unknown value (\(reason.rawValue))"
        }
    }
}
