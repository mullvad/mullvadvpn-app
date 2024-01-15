//
//  IPOverrideViewControllerDelegate.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-16.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol IPOverrideViewControllerDelegate: AnyObject {
    func controllerShouldShowTextImportView(_ controller: IPOverrideViewController)
    func controllerShouldShowFileImportView(_ controller: IPOverrideViewController)
    func controllerShouldClearAllOverrides(_ controller: IPOverrideViewController)
}
