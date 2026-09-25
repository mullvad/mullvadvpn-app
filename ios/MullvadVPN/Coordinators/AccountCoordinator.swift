// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadREST
import Routing
import SwiftUI
import UIKit

enum AccountDismissReason: Equatable, Sendable {
    case none
    case userLoggedOut
}

final class AccountCoordinator: Coordinator, Presentable, Presenting, @unchecked Sendable {
    private let tunnelManager: TunnelManager
    private let storePaymentManager: StorePaymentManager
    private let deviceManagementInteractor: DeviceManagementInteractor

    let navigationController: UINavigationController
    var presentedViewController: UIViewController {
        navigationController
    }

    var didFinish: (@MainActor (AccountCoordinator, AccountDismissReason) -> Void)?

    init(
        navigationController: UINavigationController,
        tunnelManager: TunnelManager,
        storePaymentManager: StorePaymentManager,
        deviceManagementInteractor: DeviceManagementInteractor,
    ) {
        self.navigationController = navigationController
        self.tunnelManager = tunnelManager
        self.storePaymentManager = storePaymentManager
        self.deviceManagementInteractor = deviceManagementInteractor
    }

    func start(animated: Bool) {
        let viewModel = AccountViewModel(
            tunnelManager: tunnelManager,
            onFinish: { [weak self] in
                guard let self else { return }
                self.didFinish?(self, $0)
            },
            onPaymentAction: { [weak self] in
                self?.didRequestShowInAppPurchase(paymentAction: $0)
            }
        )
        let hostingController = UIHostingRootController(
            rootView: AccountView(
                viewModel: viewModel,
                deviceManaging: deviceManagementInteractor
            )
        )
        navigationController.navigationBar.prefersLargeTitles = true
        hostingController.navigationItem.largeTitleDisplayMode = .always
        navigationController.pushViewController(hostingController, animated: false)
    }

    private func didRequestShowInAppPurchase(
        paymentAction: PaymentAction
    ) {
        guard let accountNumber = tunnelManager.deviceState.accountData?.number else { return }
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
}
