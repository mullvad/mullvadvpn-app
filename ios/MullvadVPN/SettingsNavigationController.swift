//
//  SettingsNavigationController.swift
//  MullvadVPN
//
//  Created by pronebird on 02/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Operations
import UIKit

enum SettingsNavigationRoute {
    case root
    case account
    case preferences
    case shortcuts
    case problemReport
}

enum SettingsDismissReason {
    case none
    case userLoggedOut
}

protocol SettingsNavigationControllerDelegate: AnyObject {
    func settingsNavigationController(
        _ controller: SettingsNavigationController,
        didNavigateTo route: SettingsNavigationRoute
    )

    func settingsNavigationController(
        _ controller: SettingsNavigationController,
        didFinishWithReason reason: SettingsDismissReason
    )
}

class SettingsNavigationController: CustomNavigationController, SettingsViewControllerDelegate,
    AccountViewControllerDelegate, UIAdaptivePresentationControllerDelegate
{
    private let operationQueue: AsyncOperationQueue = {
        let operationQueue = AsyncOperationQueue()
        operationQueue.maxConcurrentOperationCount = 1
        return operationQueue
    }()

    weak var settingsDelegate: SettingsNavigationControllerDelegate?

    override var childForStatusBarStyle: UIViewController? {
        return topViewController
    }

    override var childForStatusBarHidden: UIViewController? {
        return topViewController
    }

    init() {
        super.init(navigationBarClass: CustomNavigationBar.self, toolbarClass: nil)

        navigationBar.prefersLargeTitles = true

        // Navigation controller ignores `prefersLargeTitles` when using `setViewControllers()`.
        pushViewController(makeViewController(for: .root), animated: false)
    }

    required init?(coder aDecoder: NSCoder) {
        super.init(coder: aDecoder)
    }

    deinit {
        operationQueue.cancelAllOperations()
        operationQueue.isSuspended = false
    }

    override func willPop(navigationItem: UINavigationItem) {
        operationQueue.isSuspended = true
    }

    override func didPop(navigationItem: UINavigationItem) {
        if viewControllers.count == 1 {
            settingsDelegate?.settingsNavigationController(self, didNavigateTo: .root)
        }
        operationQueue.isSuspended = false
    }

    override func didBeginInteractivePop() {
        operationQueue.isSuspended = true
    }

    override func didCancelInteractivePop() {
        operationQueue.isSuspended = false
    }

    // MARK: - SettingsViewControllerDelegate

    func settingsViewControllerDidFinish(_ controller: SettingsViewController) {
        settingsDelegate?.settingsNavigationController(self, didFinishWithReason: .none)
    }

    // MARK: - AccountViewControllerDelegate

    func accountViewControllerDidLogout(_ controller: AccountViewController) {
        settingsDelegate?.settingsNavigationController(self, didFinishWithReason: .userLoggedOut)
    }

    // MARK: - Navigation

    func navigate(to route: SettingsNavigationRoute, animated: Bool) {
        let blockOperation = AsyncBlockOperation(dispatchQueue: .main) { [weak self] op in
            guard let self = self else {
                op.finish()
                return
            }

            self.navigateInner(to: route, animated: animated) {
                op.finish()
            }
        }
        operationQueue.addOperation(blockOperation)
    }

    private func navigateInner(
        to route: SettingsNavigationRoute,
        animated: Bool,
        completion: (() -> Void)?
    ) {
        if route == .root {
            popToRootViewController(animated: animated)
            notifyAnimationCompletion(completion)
        } else {
            let nextViewController = makeViewController(for: route)

            if let rootController = viewControllers.first, viewControllers.count > 1 {
                setViewControllers([rootController, nextViewController], animated: animated)
            } else {
                pushViewController(nextViewController, animated: animated)
            }

            notifyAnimationCompletion { [weak self] in
                completion?()

                if let self = self {
                    self.settingsDelegate?.settingsNavigationController(self, didNavigateTo: route)
                }
            }
        }
    }

    private func notifyAnimationCompletion(_ completion: (() -> Void)?) {
        if let transitionCoordinator = transitionCoordinator {
            transitionCoordinator.animate(alongsideTransition: nil) { _ in
                completion?()
            }
        } else {
            completion?()
        }
    }

    private func makeViewController(for route: SettingsNavigationRoute) -> UIViewController {
        switch route {
        case .root:
            let settingsController = SettingsViewController()
            settingsController.delegate = self
            return settingsController

        case .account:
            let controller = AccountViewController()
            controller.delegate = self
            return controller

        case .preferences:
            return PreferencesViewController()

        case .shortcuts:
            return ShortcutsViewController()

        case .problemReport:
            return ProblemReportViewController()
        }
    }

    // MARK: - UIAdaptivePresentationControllerDelegate

    func presentationControllerDidDismiss(_ presentationController: UIPresentationController) {
        settingsDelegate?.settingsNavigationController(self, didFinishWithReason: .none)
    }
}
