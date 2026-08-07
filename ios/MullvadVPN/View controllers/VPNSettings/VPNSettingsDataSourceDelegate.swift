//
//  VPNSettingsDataSourceDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 11/10/2021.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
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
    func showDNSSettings()
    func showIPOverrides()
    func showAntiCensorshipSettings()
    func didSelectWireGuardPort(_ port: UInt16?)
    func humanReadablePortRepresentation() -> String
    func obfuscationSettingsAreValid(
        _ settings: WireGuardObfuscationSettings,
        completion: @escaping (Bool) -> Void
    )
}
