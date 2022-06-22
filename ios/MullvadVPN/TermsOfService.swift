//
//  TermsOfService.swift
//  MullvadVPN
//
//  Created by pronebird on 22/06/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum TermsOfService {
    private static let userDefaultsKey = "isAgreedToTermsOfService"

    static var isAgreed: Bool {
        return UserDefaults.standard.bool(forKey: userDefaultsKey)
    }

    static func setAgreed() {
        UserDefaults.standard.set(true, forKey: userDefaultsKey)
    }
}
