//
//  SettingsDataSourceDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 19/10/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import UIKit

protocol SettingsDataSourceDelegate: AnyObject {
    func didSelectItem(item: SettingsDataSource.Item)
    func showInfo(for: SettingsInfoButtonItem)
}
