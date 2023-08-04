//
//  AccountFlow.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-08-03.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
enum AccountFlow {
    private static let userDefaultsKey = "isOnboarding"

    static var isOnboarding: Bool {
        set {
            UserDefaults.standard.set(newValue, forKey: userDefaultsKey)
        }
        get {
            UserDefaults.standard.bool(forKey: userDefaultsKey)
        }
    }
}
