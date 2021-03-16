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

        navigationBar.barStyle = .black
        navigationBar.tintColor = .white
        navigationBar.prefersLargeTitles = false

        (navigationBar as? CustomNavigationBar)?.prefersOpaqueBackground = true

        self.viewControllers = [contentController]
    }

    override init(nibName nibNameOrNil: String?, bundle nibBundleOrNil: Bundle?) {
        // This override has to exist to prevent crash on iOS 12 where `UINavigationController`
        // calls `self.init(nibName:bundle:)` internally.
        super.init(nibName: nibNameOrNil, bundle: nibBundleOrNil)
    }

    required init?(coder aDecoder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
