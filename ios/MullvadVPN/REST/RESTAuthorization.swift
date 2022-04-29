//
//  RESTAuthorization.swift
//  MullvadVPN
//
//  Created by pronebird on 16/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension REST {
    enum Authorization {
        case accountNumber(String)
        case accessToken(String)
    }
}
