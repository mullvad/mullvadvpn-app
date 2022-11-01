//
//  SettingsNavigationController.swift
//  MullvadVPN
//
//  Created by pronebird on 02/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

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
        willNavigateTo route: SettingsNavigationRoute
    )

    func settingsNavigationController(
        _ controller: SettingsNavigationController,
        didFinishWithReason reason: SettingsDismissReason
    )
}

class SettingsNavigationController: CustomNavigationController, SettingsViewControllerDelegate,
    AccountViewControllerDelegate, UIAdaptivePresentationControllerDelegate
{
    private let interactorFactory: SettingsInteractorFactory

    weak var settingsDelegate: SettingsNavigationControllerDelegate?

    override var childForStatusBarStyle: UIViewController? {
        return topViewController
    }

    override var childForStatusBarHidden: UIViewController? {
        return topViewController
    }

    init(interactorFactory: SettingsInteractorFactory) {
        self.interactorFactory = interactorFactory

        super.init(navigationBarClass: CustomNavigationBar.self, toolbarClass: nil)

        navigationBar.prefersLargeTitles = true

        // Navigation controller ignores `prefersLargeTitles` when using `setViewControllers()`.
        pushViewController(makeViewController(for: .root), animated: false)
    }

    required init?(coder aDecoder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func willPop(navigationItem: UINavigationItem) {
        let index = viewControllers.firstIndex { $0.navigationItem == navigationItem }

        if viewControllers.count > 1, index == 1 {
            settingsDelegate?.settingsNavigationController(self, willNavigateTo: .root)
        }
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
        guard route != .root else {
            popToRootViewController(animated: animated)
            return
        }

        settingsDelegate?.settingsNavigationController(self, willNavigateTo: route)

        let nextViewController = makeViewController(for: route)

        if let rootController = viewControllers.first, viewControllers.count > 1 {
            setViewControllers([rootController, nextViewController], animated: animated)
        } else {
            pushViewController(nextViewController, animated: animated)
        }
    }

    private func makeViewController(for route: SettingsNavigationRoute) -> UIViewController {
        switch route {
        case .root:
            let controller = SettingsViewController(
                interactor: interactorFactory.makeSettingsInteractor()
            )
            controller.delegate = self
            return controller

        case .account:
            let controller = AccountViewController(
                interactor: interactorFactory.makeAccountInteractor()
            )
            controller.delegate = self
            return controller

        case .preferences:
            return PreferencesViewController(
                interactor: interactorFactory.makePreferencesInteractor()
            )

        case .shortcuts:
            return ShortcutsViewController()

        case .problemReport:
            return ProblemReportViewController(
                interactor: interactorFactory.makeProblemReportInteractor()
            )
        }
    }

    // MARK: - UIAdaptivePresentationControllerDelegate

    func presentationControllerDidDismiss(_ presentationController: UIPresentationController) {
        settingsDelegate?.settingsNavigationController(self, didFinishWithReason: .none)
    }
}
