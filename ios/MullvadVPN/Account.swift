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

class Account {

    enum Error: Swift.Error {
        case invalidAccount
    }

    private let userDefaultsInteractor = UserDefaultsInteractor.withApplicationGroupUserDefaults()

    class func login(with accountToken: String) -> Procedure {
        let userDefaultsInteractor = UserDefaultsInteractor.withApplicationGroupUserDefaults()

        let verificationProcedure = AccountVerificationProcedure(dispatchQueue: nil, accountToken: accountToken)

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

    class func logout() {
        let userDefaultsInteractor = UserDefaultsInteractor.withApplicationGroupUserDefaults()

        userDefaultsInteractor.accountToken = nil
        userDefaultsInteractor.accountExpiry = nil
    }

}
