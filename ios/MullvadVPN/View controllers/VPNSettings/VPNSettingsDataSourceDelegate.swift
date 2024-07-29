//
//  VPNSettingsDataSourceDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 11/10/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

protocol DNSSettingsDataSourceDelegate: AnyObject {
    func didChangeViewModel(_ viewModel: VPNSettingsViewModel)
    func showInfo(for: VPNSettingsInfoButtonItem)
}

protocol VPNSettingsDataSourceDelegate: AnyObject {
    func didChangeViewModel(_ viewModel: VPNSettingsViewModel)
    func showInfo(for: VPNSettingsInfoButtonItem)
    func showDNSSettings()
    func showIPOverrides()
    func didSelectWireGuardPort(_ port: UInt16?)
}
