// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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

        navigationBar.configureCustomAppearance()
        interactivePopGestureRecognizer?.delegate = self
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()

        // Navigation bar updates the prompt color on layout so we have to force our own appearance on each layout pass.
        navigationBar.overridePromptColor()
    }
}

extension CustomNavigationController: UIGestureRecognizerDelegate {
    public func gestureRecognizerShouldBegin(_ gestureRecognizer: UIGestureRecognizer) -> Bool {
        return viewControllers.count > 1
    }

    // To make it works also with ScrollView
    public func gestureRecognizer(
        _ gestureRecognizer: UIGestureRecognizer,
        shouldRecognizeSimultaneouslyWith otherGestureRecognizer: UIGestureRecognizer
    ) -> Bool {
        true
    }
}

extension UINavigationController {
    func pushViewController(_ viewController: UIViewController, addingNavigationButtons: Bool, animated: Bool = true) {
        if addingNavigationButtons {
            viewController.navigationItem.hidesBackButton = true
            viewController.navigationItem.leftBarButtonItem = UIBarButtonItem.mullvadBackButton(
                navigationController: self)
        }
        self.pushViewController(viewController, animated: animated)
    }
}
