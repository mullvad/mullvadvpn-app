//
//  SettingsCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 09/01/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadSettings
import Operations
import Routing
import SwiftUI
import UIKit

/// Settings navigation route.
enum SettingsNavigationRoute: Equatable {
    /// The route that's always displayed first upon entering settings.
    case root

    /// VPN settings.
    case vpnSettings

    /// Problem report.
    case problemReport

    /// FAQ section displayed as a modal safari browser.
    case faq

    /// API access route.
    case apiAccess

    /// changelog route.
    case changelog

    /// Multihop route.
    case multihop

    /// DAITA route.
    case daita
}

/// Top-level settings coordinator.
@MainActor
final class SettingsCoordinator: Coordinator, Presentable, Presenting, SettingsViewControllerDelegate,
    UINavigationControllerDelegate, Sendable {
    private let logger = Logger(label: "SettingsNavigationCoordinator")

    private var currentRoute: SettingsNavigationRoute?
    nonisolated(unsafe) private var modalRoute: SettingsNavigationRoute?
    private let interactorFactory: SettingsInteractorFactory
    private var viewControllerFactory: SettingsViewControllerFactory?

    let navigationController: UINavigationController

    var presentedViewController: UIViewController {
        navigationController
    }

    /// Event handler invoked when navigating bebtween child routes within the horizontal stack.
    var willNavigate: ((
        _ coordinator: SettingsCoordinator,
        _ from: SettingsNavigationRoute?,
        _ to: SettingsNavigationRoute
    ) -> Void)?

    /// Event handler invoked when coordinator and its view hierarchy should be dismissed.
    nonisolated(unsafe) var didFinish: (@Sendable (SettingsCoordinator) -> Void)?

    /// Designated initializer.
    /// - Parameters:
    ///   - navigationController: a navigation controller that the coordinator will be managing.
    ///   - interactorFactory: an instance of a factory that produces interactors for the child routes.
    init(
        navigationController: UINavigationController,
        interactorFactory: SettingsInteractorFactory,
        accessMethodRepository: AccessMethodRepositoryProtocol,
        proxyConfigurationTester: ProxyConfigurationTesterProtocol,
        ipOverrideRepository: IPOverrideRepository
    ) {
        self.navigationController = navigationController
        self.interactorFactory = interactorFactory

        super.init()

        viewControllerFactory = SettingsViewControllerFactory(
            interactorFactory: interactorFactory,
            accessMethodRepository: accessMethodRepository,
            proxyConfigurationTester: proxyConfigurationTester,
            ipOverrideRepository: ipOverrideRepository,
            navigationController: navigationController,
            alertPresenter: AlertPresenter(context: self)
        )
    }

    /// Start the coordinator fllow.
    /// - Parameter initialRoute: the initial route to display.
    func start(initialRoute: SettingsNavigationRoute? = nil) {
        navigationController.navigationBar.prefersLargeTitles = true
        navigationController.delegate = self

        push(from: makeChild(for: .root), animated: false)
        if let initialRoute, initialRoute != .root {
            push(from: makeChild(for: initialRoute), animated: false)
        }
    }

    // MARK: - Navigation

    /// Request navigation to the speciifc route.
    ///
    /// - Parameters:
    ///   - route: the route to present.
    ///   - animated: whether transition should be animated.
    ///   - completion: a completion handler, typically called immediately for horizontal navigation and
    func navigate(
        to route: SettingsNavigationRoute,
        animated: Bool,
        completion: (@Sendable @MainActor () -> Void)? = nil
    ) {
        switch route {
        case .root:
            popToRoot(animated: animated)
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
            // Ignore navigation if the route is already presented.
            guard currentRoute != route else {
                completion?()
                return
            }

            let child = makeChild(for: route)

            // Pop to root first, then push the child.
            if navigationController.viewControllers.count > 1 {
                popToRoot(animated: animated)
            }
            push(from: child, animated: animated)

            completion?()
        }
    }

    // MARK: - UINavigationControllerDelegate

    func navigationController(
        _ navigationController: UINavigationController,
        willShow viewController: UIViewController,
        animated: Bool
    ) {
        /*
         Navigation controller calls this delegate method on `viewWillAppear`, for instance during cancellation
         of interactive dismissal of a modally presented settings navigation controller, so it's important that we
         ignore repeating routes.
         */
        guard let route = route(for: viewController), currentRoute != route else { return }

        logger.debug(
            "Navigate from \(currentRoute.map { "\($0)" } ?? "none") -> \(route)"
        )

        willNavigate?(self, currentRoute, route)

        currentRoute = route

        // Release child coordinators when moving to root.
        if case .root = route {
            releaseChildren()
        }
    }

    // MARK: - SettingsViewControllerDelegate

    nonisolated func settingsViewControllerDidFinish(_ controller: SettingsViewController) {
        didFinish?(self)
    }

    nonisolated func settingsViewController(
        _ controller: SettingsViewController,
        didRequestRoutePresentation route: SettingsNavigationRoute
    ) {
        Task {
            await navigate(to: route, animated: true)
        }
    }

    // MARK: - Route handling

    /// Pop to root route.
    /// - Parameter animated: whether to animate the transition.
    private func popToRoot(animated: Bool) {
        navigationController.popToRootViewController(animated: animated)
        releaseChildren()
    }

    /// Push the child into navigation stack.
    /// - Parameters:
    ///   - result: the result of creating a child representing a route.
    ///   - animated: whether to animate the transition.
    private func push(from result: SettingsViewControllerFactory.MakeChildResult, animated: Bool) {
        switch result {
        case let .viewController(vc):
            navigationController.pushViewController(vc, animated: animated)

        case let .childCoordinator(child):
            addChild(child)
            child.start(animated: animated)

        case .failed:
            break
        }
    }

    /// Release all child coordinators conforming to ``SettingsChildCoordinator`` protocol.
    private func releaseChildren() {
        childCoordinators.forEach { coordinator in
            if coordinator is SettingsChildCoordinator {
                coordinator.removeFromParent()
            }
        }
    }

    // MARK: - Route mapping

    /// Produce a view controller or a child coordinator representing the route.
    /// - Parameter route: the route for which to request the new view controller or child coordinator.
    /// - Returns: a result of creating a child for the route.
    private func makeChild(for route: SettingsNavigationRoute) -> SettingsViewControllerFactory.MakeChildResult {
        if route == .root {
            let controller = SettingsViewController(
                interactor: interactorFactory.makeSettingsInteractor(),
                alertPresenter: AlertPresenter(context: self)
            )
            controller.delegate = self
            return .viewController(controller)
        } else {
            return viewControllerFactory?.makeRoute(for: route) ?? .failed
        }
    }

    /// Map the view controller to the individual route.
    /// - Parameter viewController: an instance of a view controller within the navigation stack.
    /// - Returns: a route upon success, otherwise `nil`.
    private func route(for viewController: UIViewController) -> SettingsNavigationRoute? {
        switch viewController {
        case is SettingsViewController:
            return .root
        case is VPNSettingsViewController:
            return .vpnSettings
        case is ProblemReportViewController:
            return .problemReport
        case is ListAccessMethodViewController:
            return .apiAccess
        default:
            return nil
        }
    }
}
