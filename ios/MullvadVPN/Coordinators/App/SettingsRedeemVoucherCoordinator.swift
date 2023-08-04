//
//  SettingsRedeemVoucherCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-06-13.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import UIKit

final class SettingsRedeemVoucherCoordinator: Coordinator, Presentable {
    private let navigationController: UINavigationController
    private let viewController: RedeemVoucherViewController
    var didFinish: ((SettingsRedeemVoucherCoordinator) -> Void)?
    var didCancel: ((SettingsRedeemVoucherCoordinator) -> Void)?

    init(
        navigationController: UINavigationController,
        interactor: RedeemVoucherInteractor
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

extension SettingsRedeemVoucherCoordinator: RedeemVoucherViewControllerDelegate {
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

extension SettingsRedeemVoucherCoordinator: AddCreditSucceededViewControllerDelegate {
    func addCreditSucceededViewControllerDidFinish(_ controller: AddCreditSucceededViewController) {
        didFinish?(self)
    }

    func titleForAction(in controller: AddCreditSucceededViewController) -> String {
        NSLocalizedString(
            "REDEEM_VOUCHER_DISMISS_BUTTON",
            tableName: "RedeemVoucher",
            value: "Got it!",
            comment: ""
        )
    }
}
