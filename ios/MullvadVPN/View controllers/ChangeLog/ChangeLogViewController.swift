//
//  ChangeLogViewController.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-01-29.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

class ChangeLogViewController: UIHostingController<ChangeLogView<ChangeLogViewModel>> {
    override init(rootView: ChangeLogView<ChangeLogViewModel>) {
        super.init(rootView: rootView)
        view.setAccessibilityIdentifier(.changeLogAlert)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
