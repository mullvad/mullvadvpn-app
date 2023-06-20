//
//  RedeemVoucherCoordinator.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-06-13.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import UIKit

final class RedeemVoucherCoordinator: Coordinator, Presentable {
    private let navigationController: UINavigationController
    private let viewController: RedeemVoucherViewController
    var didFinish: ((RedeemVoucherCoordinator) -> Void)?
    var didCancel: ((RedeemVoucherCoordinator) -> Void)?

    init(
        navigationController: UINavigationController,
        interactor: RedeemVoucherInteractor
    ) {
        self.navigationController = navigationController
        viewController = RedeemVoucherViewController(interactor: interactor)
    }

    var presentedViewController: UIViewController {
        return navigationController
    }

    func start() {
        navigationController.navigationBar.isHidden = true
        viewController.delegate = self
        navigationController.pushViewController(viewController, animated: true)
    }
}

extension RedeemVoucherCoordinator: RedeemVoucherViewControllerDelegate {
    func redeemVoucherDidSucceed(
        _ controller: RedeemVoucherViewController,
        with response: REST.SubmitVoucherResponse
    ) {
        let viewController = RedeemVoucherSucceededViewController(timeAddedComponents: response.dateComponents)
        viewController.delegate = self
        navigationController.pushViewController(viewController, animated: true)
    }

    func redeemVoucherDidCancel(_ controller: RedeemVoucherViewController) {
        didCancel?(self)
    }
}

extension RedeemVoucherCoordinator: RedeemVoucherSucceededViewControllerDelegate {
    func redeemVoucherSucceededViewControllerDidFinish(_ controller: RedeemVoucherSucceededViewController) {
        didFinish?(self)
    }
}
