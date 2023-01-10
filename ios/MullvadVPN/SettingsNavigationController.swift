//
//  SettingsNavigationController.swift
//  MullvadVPN
//
//  Created by pronebird on 02/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import UIKit

final class SettingsNavigationController: UINavigationController {
    override var childForStatusBarStyle: UIViewController? {
        return topViewController
    }

    override var childForStatusBarHidden: UIViewController? {
        return topViewController
    }

    init() {
        super.init(navigationBarClass: CustomNavigationBar.self, toolbarClass: nil)

        navigationBar.prefersLargeTitles = true
    }

    required init?(coder aDecoder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
