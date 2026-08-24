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

        navigationBar.configureCustomAppeareance()
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()

        // Navigation bar updates the prompt color on layout so we have to force our own appearance on each layout pass.
        navigationBar.overridePromptColor()
    }
}
