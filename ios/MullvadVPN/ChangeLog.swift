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
    private static let key = "isShownChangeLogAlready"

    /// Value that indicates changes has been already presented to user.
    static var isShown: Bool {
        return UserDefaults.standard.bool(forKey: key)
    }

    /// Update `isShown` value in storage.
    static func setShown(_ isShown: Bool) {
        UserDefaults.standard.set(isShown, forKey: key)
    }
}
