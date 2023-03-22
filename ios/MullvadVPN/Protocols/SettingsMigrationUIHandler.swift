//
//  SettingsMigrationUIHandler.swift
//  MullvadVPN
//
//  Created by pronebird on 24/11/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol SettingsMigrationUIHandler {
    func showMigrationError(_ error: Error, completionHandler: @escaping () -> Void)
}
