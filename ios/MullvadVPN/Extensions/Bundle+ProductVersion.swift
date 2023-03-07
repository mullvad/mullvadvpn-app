//
//  Bundle+ProductVersion.swift
//  MullvadVPN
//
//  Created by pronebird on 22/02/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension Bundle {
    /// Returns the product version string based on the following rules:
    ///
    /// 1. Dev builds (debug): XXXX.YY-devZ
    /// 2. TestFlight builds: XXXX.YY-betaZ
    /// 3. AppStore builds: XXXX.YY
    ///
    /// Note: XXXX.YY is an app version (i.e 2020.5) and Z is a build number (i.e 1)
    var productVersion: String {
        let version = object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? "???"
        let buildNumber = object(forInfoDictionaryKey: kCFBundleVersionKey as String) as? String ??
            "???"

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
}
