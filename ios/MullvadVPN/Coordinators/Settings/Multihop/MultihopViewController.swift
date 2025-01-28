//
//  MultihopViewController.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-01-29.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

class MultihopViewController: UIHostingController<SettingsMultihopView<MultihopTunnelSettingsViewModel>> {
    override init(rootView: SettingsMultihopView<MultihopTunnelSettingsViewModel>) {
        super.init(rootView: rootView)
        view.setAccessibilityIdentifier(.multihopView)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
