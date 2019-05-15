//
//  Account.swift
//  MullvadVPN
//
//  Created by pronebird on 14/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation
import ProcedureKit
import os.log

/// Application group identifier used for sharing user settings between processes
private let kApplicationGroupIdentifier = "group.net.mullvad.MullvadVPN"

/// Account token key used for user settings storage
private let kUserDefaultsAccountTokenKey = "accountToken"

/// Account verification result
enum AccountVerification {
    /// The app should attempt to verify the account token at some point later because the network
    /// may not be available at this time.
    case deferred(Error)

    /// The app successfully verified the account token with the server
    case verified(Date)

    // Invalid token
    case invalid(Error)
}

class Account {
    class var accountToken: String? {
        return UserDefaults.mullvadUserDefaults().string(forKey: kUserDefaultsAccountTokenKey)
    }

    class func updateAccountToken(_ accountToken: String) {
        UserDefaults.mullvadUserDefaults().setValue(accountToken, forKey: kUserDefaultsAccountTokenKey)
    }

    class func verifyAccountToken(_ accountToken: String? = nil) -> AccountVerificationProcedure {
        return AccountVerificationProcedure(accountToken: accountToken)
    }
}

private extension UserDefaults {
    /// Returns the UserDefaults sharing the application group
    class func mullvadUserDefaults() -> UserDefaults {
        return UserDefaults(suiteName: kApplicationGroupIdentifier)!
    }
}
