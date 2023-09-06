//
//  AccountVoucherCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-06-13.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import Routing
import UIKit

final class AccountVoucherCoordinator: Coordinator, Presentable {
    private let navigationController: UINavigationController
    private let viewController: RedeemVoucherViewController

    var didFinish: ((AccountVoucherCoordinator) -> Void)?
    var didCancel: ((AccountVoucherCoordinator) -> Void)?

    init(
        navigationController: UINavigationController,
        interactor: AccountVoucherInteractor
    ) {
        self.navigationController = navigationController
        viewController = RedeemVoucherViewController(interactor: interactor)
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

extension AccountVoucherCoordinator: RedeemVoucherViewControllerDelegate {
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

    func viewForInputingAccountNumberAsVoucher(_ controller: RedeemVoucherViewController) -> UIView? {
        nil
    }
}

extension AccountVoucherCoordinator: AddCreditSucceededViewControllerDelegate {
    func addCreditSucceededViewControllerDidFinish(in controller: AddCreditSucceededViewController) {
        didFinish?(self)
    }

    func header(in controller: AddCreditSucceededViewController) -> String {
        NSLocalizedString(
            "REDEEM_VOUCHER_SUCCESS_TITLE",
            tableName: "AccountRedeemVoucher",
            value: "Voucher was successfully redeemed.",
            comment: ""
        )
    }

    func titleForAction(in controller: AddCreditSucceededViewController) -> String {
        NSLocalizedString(
            "REDEEM_VOUCHER_DISMISS_BUTTON",
            tableName: "AccountRedeemVoucher",
            value: "Got it!",
            comment: ""
        )
    }
}
