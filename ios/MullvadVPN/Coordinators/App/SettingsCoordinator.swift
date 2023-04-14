//
//  SettingsCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 09/01/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import Operations
import UIKit

enum SettingsNavigationRoute: Equatable {
    case root
    case preferences
    case problemReport
    case faq
}

final class SettingsCoordinator: Coordinator, Presentable, Presenting, SettingsViewControllerDelegate,
    UINavigationControllerDelegate
{
    private let logger = Logger(label: "SettingsNavigationCoordinator")

    private let interactorFactory: SettingsInteractorFactory
    private var currentRoute: SettingsNavigationRoute?
    private var modalRoute: SettingsNavigationRoute?

    let navigationController: UINavigationController

    var presentedViewController: UIViewController {
        return navigationController
    }

    var presentationContext: UIViewController {
        return navigationController
    }

    var willNavigate: ((
        _ coordinator: SettingsCoordinator,
        _ from: SettingsNavigationRoute?,
        _ to: SettingsNavigationRoute
    ) -> Void)?

    var didFinish: ((SettingsCoordinator) -> Void)?

    init(
        navigationController: UINavigationController,
        interactorFactory: SettingsInteractorFactory
    ) {
        self.navigationController = navigationController
        self.interactorFactory = interactorFactory
    }

    func start(initialRoute: SettingsNavigationRoute? = nil) {
        navigationController.navigationBar.prefersLargeTitles = true
        navigationController.delegate = self

        if let rootController = makeViewController(for: .root) {
            navigationController.pushViewController(rootController, animated: false)
        }

        if let initialRoute = initialRoute, initialRoute != .root,
           let nextController = makeViewController(for: initialRoute)
        {
            navigationController.pushViewController(nextController, animated: false)
        }
    }

    // MARK: - Navigation

    func navigate(to route: SettingsNavigationRoute, animated: Bool, completion: (() -> Void)? = nil) {
        switch route {
        case .root:
            navigationController.popToRootViewController(animated: animated)
            completion?()

        case .faq:
            guard modalRoute == nil else {
                completion?()
                return
            }

            modalRoute = route

            logger.debug("Show modal \(route)")

            let safariCoordinator = SafariCoordinator(url: ApplicationConfiguration.faqAndGuidesURL)

            safariCoordinator.didFinish = { [weak self] in
                self?.modalRoute = nil
            }

            presentChild(safariCoordinator, animated: animated, completion: completion)

        default:
            let nextViewController = makeViewController(for: route)
            let viewControllers = navigationController.viewControllers

            if let rootController = viewControllers.first, viewControllers.count > 1 {
                navigationController.setViewControllers(
                    [rootController, nextViewController].compactMap { $0 },
                    animated: animated
                )
            } else if let nextViewController = nextViewController {
                navigationController.pushViewController(nextViewController, animated: animated)
            }

            completion?()
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
        didFinish?(self)
    }

    func settingsViewController(
        _ controller: SettingsViewController,
        didRequestRoutePresentation route: SettingsNavigationRoute
    ) {
        navigate(to: route, animated: true)
    }

    // MARK: - Route mapping

    private func makeViewController(for route: SettingsNavigationRoute) -> UIViewController? {
        switch route {
        case .root:
            let controller = SettingsViewController(
                interactor: interactorFactory.makeSettingsInteractor()
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
            return nil
        }
    }

    private func route(for viewController: UIViewController) -> SettingsNavigationRoute? {
        switch viewController {
        case is SettingsViewController:
            return .root
        case is PreferencesViewController:
            return .preferences
        case is ProblemReportViewController:
            return .problemReport
        default:
            return nil
        }
    }
}
