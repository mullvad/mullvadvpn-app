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

class SettingsNavigationController: UINavigationController, SettingsViewControllerDelegate,
    AccountViewControllerDelegate, UIAdaptivePresentationControllerDelegate,
    UINavigationControllerDelegate
{
    private let interactorFactory: SettingsInteractorFactory
    private var currentRoutes: [SettingsNavigationRoute] = [.root]

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

        delegate = self
    }

    required init?(coder aDecoder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // MARK: - UINavigationControllerDelegate

    func navigationController(
        _ navigationController: UINavigationController,
        willShow viewController: UIViewController,
        animated: Bool
    ) {
        let newRoutes = viewControllers.compactMap { route(for: $0) }

        if currentRoutes != newRoutes, let nextRoute = newRoutes.last {
            currentRoutes = newRoutes
            settingsDelegate?.settingsNavigationController(self, willNavigateTo: nextRoute)
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
            let newChildren = [rootController, nextViewController]
            let newRoutes = newChildren.compactMap { self.route(for: $0) }

            currentRoutes = newRoutes
            setViewControllers(newChildren, animated: animated)
        } else {
            currentRoutes.append(route)
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

        case .problemReport:
            return ProblemReportViewController(
                interactor: interactorFactory.makeProblemReportInteractor()
            )
        }
    }

    private func route(for viewController: UIViewController) -> SettingsNavigationRoute? {
        switch viewController {
        case is SettingsViewController:
            return .root
        case is AccountViewController:
            return .account
        case is PreferencesViewController:
            return .preferences
        case is ProblemReportViewController:
            return .problemReport
        default:
            return nil
        }
    }

    // MARK: - UIAdaptivePresentationControllerDelegate

    func presentationControllerDidDismiss(_ presentationController: UIPresentationController) {
        settingsDelegate?.settingsNavigationController(self, didFinishWithReason: .none)
    }
}
