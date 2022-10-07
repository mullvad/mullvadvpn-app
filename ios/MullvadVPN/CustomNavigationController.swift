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
        assert(
            !UINavigationController.instancesRespond(
                to: #selector(UINavigationBarDelegate.navigationBar(_:didPop:))
            )
        )
    }()

    private var popGestureRecognizerDelegate: CustomPopGestureRecognizerDelegate?
    private var popItem: UINavigationItem?

    override func viewDidLoad() {
        super.viewDidLoad()

        _ = Self.classInit

        popGestureRecognizerDelegate = CustomPopGestureRecognizerDelegate(
            systemGestureRecognizerDelegate: interactivePopGestureRecognizer?.delegate,
            shouldBeginGestureRecognizer: { [weak self] gestureRecognizer in
                guard let self = self,
                      let topItem = self.navigationBar.topItem,
                      let conformingViewController = self
                      .topViewController as? ConditionalNavigation else { return true }

                return conformingViewController.shouldPopNavigationItem(
                    topItem,
                    trigger: .interactiveGesture
                )
            }
        )

        // Replace the system interactive gesture recognizer delegate
        interactivePopGestureRecognizer?.delegate = popGestureRecognizerDelegate
        interactivePopGestureRecognizer?.addTarget(
            self,
            action: #selector(interactivePopGestureDidChange)
        )
    }

    @objc private func interactivePopGestureDidChange(_ gestureRecognizer: UIGestureRecognizer) {
        switch gestureRecognizer.state {
        case .began:
            popItem = navigationBar.topItem
            didBeginInteractivePop()

        case .ended, .cancelled:
            let notifySubclass = { [weak self] in
                guard let self = self else { return }

                if self.navigationBar.topItem == self.popItem {
                    self.didCancelInteractivePop()
                }

                self.popItem = nil
            }

            if let transitionCoordinator = transitionCoordinator {
                transitionCoordinator.animate(alongsideTransition: nil, completion: { _ in
                    notifySubclass()
                })
            } else {
                notifySubclass()
            }

        default:
            break
        }
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

    func navigationBar(_ navigationBar: UINavigationBar, didPop item: UINavigationItem) {
        // UINavigationController does not override this method.
        didPop(navigationItem: item)
    }

    // MARK: - Subclass overrides

    func willPop(navigationItem: UINavigationItem) {
        // Override in subclasses
    }

    func didPop(navigationItem: UINavigationItem) {
        // Override in subclasses
    }

    func didBeginInteractivePop() {
        // Override in subclasses
    }

    func didCancelInteractivePop() {
        // Override in subclasses
    }
}

private class CustomPopGestureRecognizerDelegate: NSObject, UIGestureRecognizerDelegate {
    private let systemGestureRecognizerDelegate: UIGestureRecognizerDelegate?
    private let shouldBeginGestureRecognizer: (UIGestureRecognizer) -> Bool

    init(
        systemGestureRecognizerDelegate: UIGestureRecognizerDelegate?,
        shouldBeginGestureRecognizer: @escaping (UIGestureRecognizer) -> Bool
    ) {
        self.systemGestureRecognizerDelegate = systemGestureRecognizerDelegate
        self.shouldBeginGestureRecognizer = shouldBeginGestureRecognizer
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

        return shouldBegin && shouldBeginGestureRecognizer(gestureRecognizer)
    }
}
