//
//  Account.swift
//  MullvadVPN
//
//  Created by pronebird on 16/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation
import ProcedureKit
import os.log

/// A class that groups the account related operations
class Account {

    enum Error: Swift.Error {
        case invalidAccount
    }

    /// Perform the login and save the account token along with expiry (if available) to the
    /// application preferences.
    class func login(with accountToken: String) -> Procedure {
        let userDefaultsInteractor = UserDefaultsInteractor.withApplicationGroupUserDefaults()

        // Request account token verification
        let verificationProcedure = AccountVerificationProcedure(accountToken: accountToken)

        // Update the application preferences based on the AccountVerification result.
        let saveAccountDataProcedure = TransformProcedure { (verification) in
            switch verification {
            case .verified(let expiry):
                userDefaultsInteractor.accountToken = accountToken
                userDefaultsInteractor.accountExpiry = expiry

            case .deferred(let error):
                userDefaultsInteractor.accountToken = accountToken
                userDefaultsInteractor.accountExpiry = nil

                os_log(.info, #"Could not request the account verification "%{private}s": %{public}s"#,
                       accountToken, error.localizedDescription)

            case .invalid:
                throw Error.invalidAccount
            }
        }.injectResult(from: verificationProcedure)

        return GroupProcedure(operations: [verificationProcedure, saveAccountDataProcedure])
    }

    /// Perform the logout by erasing the account token and expiry from the application preferences.
    class func logout() {
        let userDefaultsInteractor = UserDefaultsInteractor.withApplicationGroupUserDefaults()

        userDefaultsInteractor.accountToken = nil
        userDefaultsInteractor.accountExpiry = nil
    }

}
