//
//  SettingsCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 09/01/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import Operations
import SafariServices
import UIKit

enum SettingsNavigationRoute: Equatable {
    case root
    case account
    case preferences
    case problemReport
    case faq
}

enum SettingsDismissReason: Equatable {
    case none
    case userLoggedOut
}

final class SettingsCoordinator: Coordinator, Presentable, SettingsViewControllerDelegate,
    AccountViewControllerDelegate, UINavigationControllerDelegate, SFSafariViewControllerDelegate
{
    private let logger = Logger(label: "SettingsNavigationCoordinator")

    private let interactorFactory: SettingsInteractorFactory
    private var currentRoute: SettingsNavigationRoute?

    let navigationController: UINavigationController

    var presentedViewController: UIViewController {
        return navigationController
    }

    var willNavigate: ((
        _ coordinator: SettingsCoordinator,
        _ from: SettingsNavigationRoute?,
        _ to: SettingsNavigationRoute
    ) -> Void)?

    var didFinish: ((SettingsCoordinator, SettingsDismissReason) -> Void)?

    init(
        navigationController: UINavigationController,
        interactorFactory: SettingsInteractorFactory
    ) {
        self.navigationController = navigationController
        self.interactorFactory = interactorFactory
    }

    func start() {
        navigationController.navigationBar.prefersLargeTitles = true
        navigationController.delegate = self
        navigationController.pushViewController(makeViewController(for: .root), animated: false)
    }

    // MARK: - Navigation

    func navigate(
        to route: SettingsNavigationRoute,
        animated: Bool,
        completion: (() -> Void)? = nil
    ) {
        switch route {
        case .root:
            navigationController.popToRootViewController(animated: animated)

        case .faq:
            let safariController = makeViewController(for: route)

            navigationController.present(safariController, animated: true)

        default:
            let nextViewController = makeViewController(for: route)
            let viewControllers = navigationController.viewControllers

            if let rootController = viewControllers.first, viewControllers.count > 1 {
                navigationController.setViewControllers(
                    [rootController, nextViewController],
                    animated: animated
                )
            } else {
                navigationController.pushViewController(nextViewController, animated: animated)
            }
        }
    }

    // MARK: - UINavigationControllerDelegate

    func navigationController(
        _ navigationController: UINavigationController,
        willShow viewController: UIViewController,
        animated: Bool
    ) {
        guard let route = route(for: viewController) else { return }

        logger.debug(
            "Navigate from \(currentRoute.map { "\($0)" } ?? "none") -> \(route)"
        )

        willNavigate?(self, currentRoute, route)

        currentRoute = route
    }

    // MARK: - SettingsViewControllerDelegate

    func settingsViewControllerDidFinish(_ controller: SettingsViewController) {
        didFinish?(self, .none)
    }

    func settingsViewController(
        _ controller: SettingsViewController,
        didRequestRoutePresentation route: SettingsNavigationRoute
    ) {
        navigate(to: route, animated: true)
    }

    // MARK: - AccountViewControllerDelegate

    func accountViewControllerDidLogout(_ controller: AccountViewController) {
        didFinish?(self, .userLoggedOut)
    }

    // MARK: - SFSafariViewControllerDelegate

    func safariViewControllerWillOpenInBrowser(_ controller: SFSafariViewController) {
        controller.dismiss(animated: false)
    }

    func safariViewControllerDidFinish(_ controller: SFSafariViewController) {
        controller.dismiss(animated: true)
    }

    // MARK: - Route mapping

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

        case .faq:
            let safariController = SFSafariViewController(
                url: ApplicationConfiguration
                    .faqAndGuidesURL
            )
            safariController.delegate = self
            return safariController
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
}
