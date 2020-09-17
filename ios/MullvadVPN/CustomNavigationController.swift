//
//  CustomNavigationController.swift
//  MullvadVPN
//
//  Created by pronebird on 17/09/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

enum NavigationPopTrigger {
    case backButton
    case interactiveGesture
}

protocol ConditionalNavigation: class {
    func shouldPopNavigationItem(_ navigationItem: UINavigationItem, trigger: NavigationPopTrigger) -> Bool
}

class CustomNavigationController: UINavigationController, UINavigationBarDelegate, UIGestureRecognizerDelegate {

    override func viewDidLoad() {
        super.viewDidLoad()

        // Since we take over the system delegate, we have to handle a couple of edge cases
        // in `gestureRecognizerShouldBegin`
        interactivePopGestureRecognizer?.delegate = self
    }

    func navigationBar(_ navigationBar: UINavigationBar, shouldPop item: UINavigationItem) -> Bool {
        if let conformingViewController = topViewController as? ConditionalNavigation {
            return conformingViewController.shouldPopNavigationItem(item, trigger: .backButton)
        }

        return true
    }

    func gestureRecognizerShouldBegin(_ gestureRecognizer: UIGestureRecognizer) -> Bool {
        guard gestureRecognizer == interactivePopGestureRecognizer else { return true }

        // Ignore gesture recognition during animations
        if let transitionCoordinator = transitionCoordinator, transitionCoordinator.isAnimated {
            return false
        }

        // Ignore gesture recognition with less than two controllers on stack
        if viewControllers.count < 2 {
            return false
        }

        if let topItem = navigationBar.topItem, let conformingViewController = topViewController as? ConditionalNavigation {
            return conformingViewController.shouldPopNavigationItem(topItem, trigger: .interactiveGesture)
        }

        return true
    }
}
