//
//  FirstTimeLaunch.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-04-04.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum FirstTimeLaunch {
    private static let userDefaultsKey = "hasFinishedFirstTimeLaunch"

    static let compatibleAppVersion = "2023.3"

    static var hasFinished: Bool {
        return UserDefaults.standard.bool(forKey: userDefaultsKey)
    }

    static func setHasFinished() {
        UserDefaults.standard.set(true, forKey: userDefaultsKey)
    }
}
