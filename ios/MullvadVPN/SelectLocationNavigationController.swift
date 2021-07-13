//
//  SelectLocationNavigationController.swift
//  MullvadVPN
//
//  Created by pronebird on 22/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class SelectLocationNavigationController: UINavigationController {

    init(contentController: SelectLocationViewController) {
        super.init(navigationBarClass: CustomNavigationBar.self, toolbarClass: nil)

        self.viewControllers = [contentController]
    }

    override init(nibName nibNameOrNil: String?, bundle nibBundleOrNil: Bundle?) {
        // This initializer exists to prevent crash on iOS 12.
        // See: https://stackoverflow.com/a/38335090/351305
        super.init(nibName: nibNameOrNil, bundle: nibBundleOrNil)
    }

    required init?(coder aDecoder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
