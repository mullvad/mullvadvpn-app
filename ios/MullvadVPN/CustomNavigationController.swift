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

class CustomNavigationController: UINavigationController, UINavigationBarDelegate {

    private var popGestureRecognizerDelegate: CustomPopGestureRecognizerDelegate?

    override func viewDidLoad() {
        super.viewDidLoad()

        popGestureRecognizerDelegate = CustomPopGestureRecognizerDelegate(navigationController: self, systemGestureRecognizerDelegate: interactivePopGestureRecognizer?.delegate)

        // Replace the system interactive gesture recognizer
        interactivePopGestureRecognizer?.delegate = popGestureRecognizerDelegate
    }

    func navigationBar(_ navigationBar: UINavigationBar, shouldPop item: UINavigationItem) -> Bool {
        if let conformingViewController = topViewController as? ConditionalNavigation {
            return conformingViewController.shouldPopNavigationItem(item, trigger: .backButton)
        }

        return true
    }
}

private class CustomPopGestureRecognizerDelegate: NSObject, UIGestureRecognizerDelegate {

    private let systemGestureRecognizerDelegate: UIGestureRecognizerDelegate?
    private weak var navigationController: UINavigationController?

    init(navigationController: UINavigationController, systemGestureRecognizerDelegate: UIGestureRecognizerDelegate?) {
        self.navigationController = navigationController
        self.systemGestureRecognizerDelegate = systemGestureRecognizerDelegate
    }

    override func responds(to aSelector: Selector!) -> Bool {
        if Self.instancesRespond(to: aSelector) {
            return true
        } else {
            return systemGestureRecognizerDelegate?.responds(to: aSelector) ?? false
        }
    }

    override func forwardingTarget(for aSelector: Selector!) -> Any? {
        let shouldForward = systemGestureRecognizerDelegate?.responds(to: aSelector) ?? false

        if shouldForward {
            return systemGestureRecognizerDelegate
        } else {
            return nil
        }
    }

    // MARK: - UIGestureRecognizerDelegate

    func gestureRecognizerShouldBegin(_ gestureRecognizer: UIGestureRecognizer) -> Bool {
        let shouldBegin = systemGestureRecognizerDelegate?.gestureRecognizerShouldBegin?(gestureRecognizer) ?? true
        
        guard let navigationController = navigationController,
            let topItem = navigationController.navigationBar.topItem,
            let conformingViewController = navigationController.topViewController as? ConditionalNavigation else {
                return shouldBegin
        }
        
        return shouldBegin && conformingViewController.shouldPopNavigationItem(topItem, trigger: .interactiveGesture)
    }
}
