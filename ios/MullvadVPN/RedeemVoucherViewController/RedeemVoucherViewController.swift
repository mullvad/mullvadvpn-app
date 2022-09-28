//
//  RedeemVoucherViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 28/09/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import UIKit

protocol RedeemVoucherViewControllerDelegate: AnyObject {
    func redeemVoucherViewControllerDidFinish(_ controller: RedeemVoucherViewController)
    func redeemVoucherViewControllerDidCancel(_ controller: RedeemVoucherViewController)
}

class RedeemVoucherViewController: UINavigationController, UINavigationControllerDelegate,
    RedeemVoucherInputViewControllerDelegate, RedeemVoucherSucceededViewControllerDelegate
{
    weak var redeemVoucherDelegate: RedeemVoucherViewControllerDelegate?

    init() {
        super.init(nibName: nil, bundle: nil)

        delegate = self
        isNavigationBarHidden = true
        preferredContentSize = CGSize(width: 450, height: 316)

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
            timeAddedComponents: response
                .dateComponents
        )
        controller.delegate = self

        pushViewController(controller, animated: true)
    }

    func redeemVoucherInputViewControllerDidCancel(_ controller: RedeemVoucherInputViewController) {
        redeemVoucherDelegate?.redeemVoucherViewControllerDidFinish(self)
    }

    // MARK: - RedeemVoucherSucceededViewControllerDelegate

    func redeemVoucherSucceededViewControllerDidFinish(
        _ controller: RedeemVoucherSucceededViewController
    ) {
        redeemVoucherDelegate?.redeemVoucherViewControllerDidCancel(self)
    }
}
