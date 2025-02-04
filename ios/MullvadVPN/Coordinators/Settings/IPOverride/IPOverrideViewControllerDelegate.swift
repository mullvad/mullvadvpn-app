//
//  IPOverrideViewControllerDelegate.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-16.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol IPOverrideViewControllerDelegate: AnyObject {
    func presentImportTextController()
    func presentAbout()
}
