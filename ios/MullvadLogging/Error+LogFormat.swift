//
//  Error+LogFormat.swift
//  MullvadLogging
//
//  Created by pronebird on 26/09/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

extension Error {
    public var description: String {
        (self as NSError).description
    }
}
