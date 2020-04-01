//
//  LoginState.swift
//  MullvadVPN
//
//  Created by pronebird on 21/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum AuthenticationMethod {
    case existingAccount, newAccount
}

enum LoginState {
    case `default`
    case authenticating(AuthenticationMethod)
    case failure(AccountError)
    case success(AuthenticationMethod)
}
