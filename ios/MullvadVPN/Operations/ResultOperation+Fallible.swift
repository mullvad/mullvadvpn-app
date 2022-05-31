//
//  ResultOperation+Fallible.swift
//  MullvadVPN
//
//  Created by pronebird on 31/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension ResultOperation: FallibleOperation {
    var error: Error? {
        return completion?.error
    }
}
