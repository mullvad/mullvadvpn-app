//
//  CustomNavigationController.swift
//  MullvadVPN
//
//  Created by pronebird on 23/02/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

class CustomNavigationController: UINavigationController {
    override var childForStatusBarHidden: UIViewController? {
        return topViewController
    }

    override var childForStatusBarStyle: UIViewController? {
        return topViewController
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        navigationBar.configureCustomAppeareance()
    }
}
