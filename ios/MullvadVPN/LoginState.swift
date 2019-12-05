//
//  LoginState.swift
//  MullvadVPN
//
//  Created by pronebird on 21/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation

enum LoginState {
    case `default`
    case authenticating
    case failure(AccountError)
    case success
}
