//
//  TermsOfServiceCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 29/01/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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
