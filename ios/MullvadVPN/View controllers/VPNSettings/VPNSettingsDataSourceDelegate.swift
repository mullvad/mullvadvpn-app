//
//  VPNSettingsDataSourceDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 11/10/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

protocol DNSSettingsDataSourceDelegate: AnyObject {
    func didChangeViewModel(_ viewModel: VPNSettingsViewModel)
    func showInfo(for: VPNSettingsInfoButtonItem)
}

protocol VPNSettingsDataSourceDelegate: AnyObject {
    func didUpdateTunnelSettings(_ update: TunnelSettingsUpdate)
    func showInfo(for: VPNSettingsInfoButtonItem)
    func showDetails(for: VPNSettingsDetailsButtonItem)
    func showDNSSettings()
    func showIPOverrides()
    func didSelectWireGuardPort(_ port: UInt16?)
    func humanReadablePortRepresentation() -> String
    func showLocalNetworkSharingWarning(_ enable: Bool, completion: @escaping (Bool) -> Void)
}
