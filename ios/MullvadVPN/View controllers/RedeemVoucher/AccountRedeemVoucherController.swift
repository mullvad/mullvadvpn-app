//
//  AccountRedeemVoucherController.swift
//  MullvadVPN
//
//  Created by pronebird on 28/09/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import UIKit

private extension UIMetrics {
    static let preferredContentSize = CGSize(width: 320, height: 310)
}

protocol AccountRedeemVoucherControllerDelegate: AnyObject {
    func redeemVoucherControllerDidFinish(_ controller: AccountRedeemVoucherController)
    func redeemVoucherControllerDidCancel(_ controller: AccountRedeemVoucherController)
}

class AccountRedeemVoucherController: UINavigationController, UINavigationControllerDelegate {
    weak var redeemVoucherDelegate: AccountRedeemVoucherControllerDelegate?
    private var customTransitioningDelegate = FormSheetTransitioningDelegate()

    init(interactor: RedeemVoucherInteractor) {
        super.init(nibName: nil, bundle: nil)
        delegate = self
        configureUI()
        setupContentView(interactor: interactor)
    }

    required init?(coder aDecoder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // MARK: - Setup

    private func setupContentView(interactor: RedeemVoucherInteractor) {
        let redeemVoucherViewController = RedeemVoucherViewController(interactor: interactor)
        redeemVoucherViewController.delegate = self
        pushViewController(redeemVoucherViewController, animated: false)
    }

    private func configureUI() {
        isNavigationBarHidden = true
        preferredContentSize = UIMetrics.preferredContentSize
        modalPresentationStyle = .custom
        modalTransitionStyle = .crossDissolve
        transitioningDelegate = customTransitioningDelegate
    }
}

extension AccountRedeemVoucherController: RedeemVoucherViewControllerDelegate {
    func redeemVoucherDidSuccess(
        _ controller: RedeemVoucherViewController,
        with response: MullvadREST.REST.SubmitVoucherResponse
    ) {
        let controller = RedeemVoucherSucceededViewController(timeAddedComponents: response.dateComponents)
        controller.delegate = self
        pushViewController(controller, animated: true)
    }

    func redeemVoucherDidCancel(_ controller: RedeemVoucherViewController) {
        controller.dismiss(animated: true)
    }
}

extension AccountRedeemVoucherController: RedeemVoucherSucceededViewControllerDelegate {
    func redeemVoucherSucceededViewControllerDidFinish(_ controller: RedeemVoucherSucceededViewController) {
        controller.dismiss(animated: true)
    }
}
