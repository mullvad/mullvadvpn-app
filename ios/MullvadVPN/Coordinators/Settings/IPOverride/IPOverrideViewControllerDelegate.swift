//
//  IPOverrideViewControllerDelegate.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-16.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

@MainActor
protocol IPOverrideViewControllerDelegate: AnyObject {
    func presentImportTextController()
    func presentAbout()
}
