//
//  InterceptibleNavigationController.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-04-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

class InterceptibleNavigationController: CustomNavigationController {
    var shouldPopViewController: ((UIViewController) -> Bool)?
    var shouldPopToViewController: ((UIViewController) -> Bool)?

    // Called when popping the topmost view controller in the stack, eg. by pressing a navigation
    // bar back button.
    @discardableResult
    override func popViewController(animated: Bool) -> UIViewController? {
        guard let viewController = viewControllers.last else { return nil }

        if shouldPopViewController?(viewController) ?? true {
            return super.popViewController(animated: animated)
        } else {
            return nil
        }
    }

    // Called when popping to a specific view controller, eg. by long pressing a navigation bar
    // back button (revealing a navigation menu) and selecting a destination view controller.
    @discardableResult
    override func popToViewController(_ viewController: UIViewController, animated: Bool) -> [UIViewController]? {
        if shouldPopToViewController?(viewController) ?? true {
            return super.popToViewController(viewController, animated: animated)
        } else {
            return nil
        }
    }
}
