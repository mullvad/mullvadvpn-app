//
//  PreferencesDataSourceDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 11/10/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol PreferencesDataSourceDelegate: AnyObject {
    func didChangeViewModel(_ viewModel: PreferencesViewModel)
    func showInfo(for: PreferencesInfoButtonItem)
    func showDNSSettings()
    func didSelectWireGuardPort(_ port: UInt16?)
}
