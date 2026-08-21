// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadREST
import Routing
import UIKit

final class ProfileVoucherCoordinator: Coordinator, Presentable {
    private let navigationController: UINavigationController
    private let viewController: RedeemVoucherViewController

    var didFinish: ((ProfileVoucherCoordinator) -> Void)?
    var didCancel: ((ProfileVoucherCoordinator) -> Void)?

    init(
        navigationController: UINavigationController,
        interactor: RedeemVoucherInteractor
    ) {
        self.navigationController = navigationController
        viewController = RedeemVoucherViewController(
            configuration: RedeemVoucherViewConfiguration(
                adjustViewWhenKeyboardAppears: false,
                shouldUseCompactStyle: true,
                layoutMargins: UIMetrics.SettingsRedeemVoucher.contentLayoutMargins
            ),
            interactor: interactor
        )
    }

    var presentedViewController: UIViewController {
        navigationController
    }

    func start() {
        navigationController.navigationBar.isHidden = true
        viewController.delegate = self
        navigationController.pushViewController(viewController, animated: true)
    }
}

extension ProfileVoucherCoordinator: @preconcurrency RedeemVoucherViewControllerDelegate {
    func redeemVoucherDidSucceed(
        _ controller: RedeemVoucherViewController,
        with response: REST.SubmitVoucherResponse
    ) {
        let viewController = AddCreditSucceededViewController(timeAddedComponents: response.dateComponents)
        viewController.delegate = self
        navigationController.pushViewController(viewController, animated: true)
    }

    func redeemVoucherDidCancel(_ controller: RedeemVoucherViewController) {
        didCancel?(self)
    }
}

extension ProfileVoucherCoordinator: @preconcurrency AddCreditSucceededViewControllerDelegate {
    func addCreditSucceededViewControllerDidFinish(in controller: AddCreditSucceededViewController) {
        didFinish?(self)
    }

    func header(in controller: AddCreditSucceededViewController) -> String {
        NSLocalizedString("Voucher was successfully redeemed.", comment: "")
    }

    func titleForAction(in controller: AddCreditSucceededViewController) -> String {
        NSLocalizedString("Got it!", comment: "")
    }
}
