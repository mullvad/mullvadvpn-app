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

protocol ConditionalNavigation: AnyObject {
    func shouldPopNavigationItem(_ navigationItem: UINavigationItem, trigger: NavigationPopTrigger)
        -> Bool
}

class CustomNavigationController: UINavigationController, UINavigationBarDelegate {
    private static let classInit: Void = {
        let isSwizzled = swizzleMethod(
            aClass: CustomNavigationController.self,
            originalSelector: #selector(UINavigationBarDelegate.navigationBar(_:shouldPop:)),
            newSelector: #selector(customNavigationController_navigationBar(_:shouldPop:))
        )
        assert(isSwizzled)
    }()

    private var popGestureRecognizerDelegate: CustomPopGestureRecognizerDelegate?

    override func viewDidLoad() {
        super.viewDidLoad()

        _ = Self.classInit

        popGestureRecognizerDelegate = CustomPopGestureRecognizerDelegate(
            navigationController: self,
            systemGestureRecognizerDelegate: interactivePopGestureRecognizer?.delegate
        )

        // Replace the system interactive gesture recognizer
        interactivePopGestureRecognizer?.delegate = popGestureRecognizerDelegate
    }

    @objc dynamic func customNavigationController_navigationBar(
        _ navigationBar: UINavigationBar,
        shouldPop item: UINavigationItem
    ) -> Bool {
        var shouldPop = true

        if let conformingViewController = topViewController as? ConditionalNavigation {
            shouldPop = conformingViewController.shouldPopNavigationItem(item, trigger: .backButton)
        }

        // Only call super implementation when we want to pop the controller
        if shouldPop {
            willPop(navigationItem: item)

            // Call super implementation
            return customNavigationController_navigationBar(navigationBar, shouldPop: item)
        } else {
            return shouldPop
        }
    }

    func willPop(navigationItem: UINavigationItem) {
        // Override in subclasses
    }
}

private class CustomPopGestureRecognizerDelegate: NSObject, UIGestureRecognizerDelegate {
    private let systemGestureRecognizerDelegate: UIGestureRecognizerDelegate?
    private weak var navigationController: UINavigationController?

    init(
        navigationController: UINavigationController,
        systemGestureRecognizerDelegate: UIGestureRecognizerDelegate?
    ) {
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
        let shouldBegin = systemGestureRecognizerDelegate?
            .gestureRecognizerShouldBegin?(gestureRecognizer) ?? true

        guard let navigationController = navigationController,
              let topItem = navigationController.navigationBar.topItem,
              let conformingViewController = navigationController
              .topViewController as? ConditionalNavigation
        else {
            return shouldBegin
        }

        return shouldBegin && conformingViewController.shouldPopNavigationItem(
            topItem,
            trigger: .interactiveGesture
        )
    }
}
