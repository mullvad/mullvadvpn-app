//
//  PreferencesDataSourceDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 11/10/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol PreferencesDataSourceDelegate: AnyObject {
    func preferencesDataSource(
        _ dataSource: PreferencesDataSource,
        didChangeViewModel viewModel: PreferencesViewModel
    )

    func preferencesDataSource(
        _ dataSource: PreferencesDataSource,
        showInfo for: PreferencesDataSource.InfoButtonItem?
    )

    func preferencesDataSource(
        _ dataSource: PreferencesDataSource,
        didSelectPort port: UInt16?
    )
}
