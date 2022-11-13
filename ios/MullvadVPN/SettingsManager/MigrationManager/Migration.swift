//
//  Migration.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-11-18.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol Migration {
    func migrate(
        with middleware: SettingsStorageMiddleware,
        completion: @escaping (Error?) -> Void
    )
}
