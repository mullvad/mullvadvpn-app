//
//  Navigator.swift
//  MullvadVPN
//
//  Created by pronebird on 12/01/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

class Navigator: NSObject, UINavigationControllerDelegate {
    private var animationDidFinish: (() -> Void)?

    let navigationController: UINavigationController
    var willShow: ((UIViewController) -> Void)?

    var children: [UIViewController] {
        return navigationController.viewControllers
    }

    init(navigationController: UINavigationController) {
        self.navigationController = navigationController
        super.init()

        navigationController.delegate = self
    }

    func replace(_ children: [UIViewController], animated: Bool, completion: (() -> Void)? = nil) {
        animationDidFinish = completion
        navigationController.setViewControllers(children, animated: animated)
    }

    func push(_ child: UIViewController, animated: Bool, completion: (() -> Void)? = nil) {
        animationDidFinish = completion
        navigationController.pushViewController(child, animated: animated)
    }

    func popToRoot(animated: Bool, completion: (() -> Void)? = nil) {
        animationDidFinish = completion
        navigationController.popToRootViewController(animated: animated)
    }

    // MARK: - UINavigationControllerDelegate

    func navigationController(
        _ navigationController: UINavigationController,
        willShow viewController: UIViewController,
        animated: Bool
    ) {
        willShow?(viewController)
    }

    func navigationController(
        _ navigationController: UINavigationController,
        didShow viewController: UIViewController,
        animated: Bool
    ) {
        let completionHandler = animationDidFinish
        animationDidFinish = nil

        completionHandler?()
    }
}
