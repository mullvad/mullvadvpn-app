//
//  CustomNavigationController.swift
//  MullvadVPN
//
//  Created by pronebird on 23/02/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// Custom navigation controller that applies the custom appearance to itself.
class CustomNavigationController: UINavigationController {
    override var childForStatusBarHidden: UIViewController? {
        topViewController
    }

    override var childForStatusBarStyle: UIViewController? {
        topViewController
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        navigationBar.configureCustomAppeareance()
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()

        // Navigation bar updates the prompt color on layout so we have to force our own appearance on each layout pass.
        navigationBar.overridePromptColor()
    }
}
