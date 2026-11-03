// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadSettings
import Routing
import SwiftUI
import UIKit

class OutOfTimeCoordinator: Coordinator, Presenting, Poppable {
    let navigationController: RootContainerViewController
    let storePaymentManager: StorePaymentManager
    let tunnelManager: TunnelManager

    var didFinishPayment: ((OutOfTimeCoordinator) -> Void)?

    var presentedViewController: UIViewController {
        navigationController
    }

    private var viewController: UIHostingRootController<OutOfTimeView<OutOfTimeViewModel>>?

    init(
        navigationController: RootContainerViewController,
        storePaymentManager: StorePaymentManager,
        tunnelManager: TunnelManager
    ) {
        self.navigationController = navigationController
        self.storePaymentManager = storePaymentManager
        self.tunnelManager = tunnelManager
    }

    func start(animated: Bool) {
        let viewModel = OutOfTimeViewModel(interactor: OutOfTimeInteractor(tunnelManager: tunnelManager))
        viewModel.showInAppPurchase = { [weak self] action in
            guard let accountNumber = self?.tunnelManager.deviceState.accountData?.number else {
                return
            }
            self?.showInAppPurchase(
                accountNumber: accountNumber,
                paymentAction: action
            )
        }

        viewModel.didAddMoreCredit = { [weak self] in
            guard let self else { return }
            didFinishPayment?(self)
        }

        viewModel.didReceiveTunnelStatus = { [weak self] in
            self?.updateHeaderBarPresentation()
        }

        let view = OutOfTimeView(viewModel: viewModel)
        let controller = UIHostingRootController(rootView: view)
        controller.view.setAccessibilityIdentifier(.outOfTimeView)
        viewController = controller

        navigationController.pushViewController(controller, animated: animated)
        updateHeaderBarPresentation()
    }

    func popFromNavigationStack(animated: Bool, completion: (() -> Void)?) {
        guard let viewController else {
            completion?()
            return
        }

        let viewControllers = navigationController.viewControllers.filter { $0 != viewController }

        navigationController.setViewControllers(
            viewControllers,
            animated: animated,
            completion: completion
        )
    }

    func showInAppPurchase(
        accountNumber: String,
        paymentAction: PaymentAction
    ) {
        let coordinator = InAppPurchaseCoordinator(
            storePaymentManager: storePaymentManager,
            accountNumber: accountNumber,
            paymentAction: paymentAction
        )
        coordinator.didFinish = { coordinator in
            coordinator.dismiss(animated: true)
        }
        coordinator.start()
        presentChild(coordinator, animated: true)
    }

    private func updateHeaderBarPresentation() {
        viewController?.prefersDeviceInfoBarHidden = false
        viewController?.preferredHeaderBarPresentation =
            switch tunnelManager.deviceState {
            case .loggedIn, .revoked:
                HeaderBarPresentation(
                    style: tunnelManager.tunnelStatus.state.isSecured ? .secured : .unsecured,

                )
            case .loggedOut:
                HeaderBarPresentation(style: .default)
            }

        navigationController.updateHeaderDeviceHiddenAppearance()
        navigationController.updateHeaderBarAppearance()
    }
}
