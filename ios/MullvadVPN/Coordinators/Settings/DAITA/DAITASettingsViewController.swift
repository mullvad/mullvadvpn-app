//
//  DAITAViewController.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-01-28.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

class DAITASettingsViewController: UIHostingController<SettingsDAITAView<DAITATunnelSettingsViewModel>> {
    override init(rootView: SettingsDAITAView<DAITATunnelSettingsViewModel>) {
        super.init(rootView: rootView)
        view.setAccessibilityIdentifier(.daitaView)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
