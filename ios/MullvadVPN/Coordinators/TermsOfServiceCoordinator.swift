// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Routing
import SwiftUI
import UIKit

class TermsOfServiceCoordinator: Coordinator, Presenting {
    private let navigationController: RootContainerViewController

    var presentationContext: UIViewController {
        navigationController
    }

    var didAgreeToTermsOfService: (() -> Void)?

    init(navigationController: RootContainerViewController) {
        self.navigationController = navigationController
    }

    func start() {
        let termsOfService = TermsOfServiceView(agreeToTermsAndServices: didAgreeToTermsOfService)
        let hostingController = UIHostingRootController(rootView: termsOfService)
        hostingController.view.setAccessibilityIdentifier(.termsOfServiceView)
        navigationController.pushViewController(hostingController, animated: false)
    }
}
