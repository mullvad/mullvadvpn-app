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

extension Bundle {
    /// Returns the product version string based on the following rules:
    ///
    /// 1. Dev builds (debug): XXXX.YY-devZ
    /// 2. TestFlight builds: XXXX.YY-betaZ
    /// 3. App Store builds: XXXX.YY
    ///
    /// Note: XXXX.YY is an app version (i.e 2020.5) and Z is a build number (i.e 1)
    public var productVersion: String {
        let version = object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? "???"
        let buildNumber = object(forInfoDictionaryKey: kCFBundleVersionKey as String) as? String ?? "???"

        #if DEBUG
            return "\(version)-dev\(buildNumber)"
        #else
            if appStoreReceiptURL?.lastPathComponent == "sandboxReceipt" {
                return "\(version)-beta\(buildNumber)"
            } else {
                return version
            }
        #endif
    }

    /// Returns short version XXXX.YY (i.e 2020.5).
    public var shortVersion: String {
        object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? "???"
    }
}
