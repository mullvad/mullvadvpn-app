//
//  RedeemVoucherController.swift
//  MullvadVPN
//
//  Created by pronebird on 28/09/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import UIKit

protocol RedeemVoucherControllerDelegate: AnyObject {
    func redeemVoucherControllerDidFinish(_ controller: RedeemVoucherController)
    func redeemVoucherControllerDidCancel(_ controller: RedeemVoucherController)
}

class RedeemVoucherController: UINavigationController, UINavigationControllerDelegate,
    RedeemVoucherInputViewControllerDelegate, RedeemVoucherSucceededViewControllerDelegate
{
    weak var redeemVoucherDelegate: RedeemVoucherControllerDelegate?

    init() {
        super.init(nibName: nil, bundle: nil)

        delegate = self
        isNavigationBarHidden = true
        preferredContentSize = CGSize(width: 450, height: 300)

        let inputController = RedeemVoucherInputViewController()
        inputController.delegate = self

        pushViewController(inputController, animated: false)
    }

    required init?(coder aDecoder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // MARK: - RedeemVoucherInputViewControllerDelegate

    func redeemVoucherInputViewController(
        _ controller: RedeemVoucherInputViewController,
        didRedeemVoucherWithResponse response: REST.SubmitVoucherResponse
    ) {
        let controller = RedeemVoucherSucceededViewController(
            timeAddedComponents: response.dateComponents
        )
        controller.delegate = self

        pushViewController(controller, animated: true)
    }

    func redeemVoucherInputViewControllerDidCancel(_ controller: RedeemVoucherInputViewController) {
        redeemVoucherDelegate?.redeemVoucherControllerDidCancel(self)
    }

    // MARK: - RedeemVoucherSucceededViewControllerDelegate

    func redeemVoucherSucceededViewControllerDidFinish(
        _ controller: RedeemVoucherSucceededViewController
    ) {
        redeemVoucherDelegate?.redeemVoucherControllerDidFinish(self)
    }

    // MARK: - UINavigationControllerDelegate

    func navigationController(
        _ navigationController: UINavigationController,
        animationControllerFor operation: UINavigationController.Operation,
        from fromVC: UIViewController,
        to toVC: UIViewController
    ) -> UIViewControllerAnimatedTransitioning? {
        if operation == .push {
            return NavigationControllerFadeAnimator()
        }
        return nil
    }
}
