//
//  ChangeLog.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-12-18.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum ChangeLog {
    /// Key to store in storage.
    private static let key = "changeLogPresentedForVersion"

    /// Value that indicates changes has been already presented to user.
    static var isShown: Bool {
        let appVersion = Bundle.main.releaseVersionNumber
        let storedVersion = UserDefaults.standard.integer(forKey: key)

        if appVersion <= storedVersion {
            return true
        }

        return false
    }

    /// Update version value in storage.
    static func setVersion(_ version: Int = Bundle.main.releaseVersionNumber) {
        UserDefaults.standard.set(version, forKey: key)
    }
}

extension Bundle {
    /// Product version integer value.
    /// - Warning: It removes all the characters expect decimal digits.
    fileprivate var releaseVersionNumber: Int {
        return Int(
            Bundle.main.productVersion
                .components(separatedBy: NSCharacterSet.decimalDigits.inverted)
                .joined(separator: "")
        ) ?? 1
    }
}
