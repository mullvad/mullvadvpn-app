//
//  SettingsCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 09/01/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
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

    /// Multihop route.
    case multihop

    /// DAITA route.
    case daita
}

/// Top-level settings coordinator.
final class SettingsCoordinator: Coordinator, Presentable, Presenting, SettingsViewControllerDelegate,
    UINavigationControllerDelegate {
    private let logger = Logger(label: "SettingsNavigationCoordinator")

    private let interactorFactory: SettingsInteractorFactory
    private var currentRoute: SettingsNavigationRoute?
    private var modalRoute: SettingsNavigationRoute?
    private let accessMethodRepository: AccessMethodRepositoryProtocol
    private let proxyConfigurationTester: ProxyConfigurationTesterProtocol
    private let ipOverrideRepository: IPOverrideRepository

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
    var didFinish: ((SettingsCoordinator) -> Void)?

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
        self.accessMethodRepository = accessMethodRepository
        self.proxyConfigurationTester = proxyConfigurationTester
        self.ipOverrideRepository = ipOverrideRepository
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
    func navigate(to route: SettingsNavigationRoute, animated: Bool, completion: (() -> Void)? = nil) {
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

    func settingsViewControllerDidFinish(_ controller: SettingsViewController) {
        didFinish?(self)
    }

    func settingsViewController(
        _ controller: SettingsViewController,
        didRequestRoutePresentation route: SettingsNavigationRoute
    ) {
        navigate(to: route, animated: true)
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
    private func push(from result: MakeChildResult, animated: Bool) {
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

    /// The result of creating a child representing a route.
    private enum MakeChildResult {
        /// View controller that should be pushed into navigation stack.
        case viewController(UIViewController)

        /// Child coordinator that should be added to the children hierarchy.
        /// The child is responsile for presenting itself.
        case childCoordinator(SettingsChildCoordinator)

        /// Failure to produce a child.
        case failed
    }

    /// Produce a view controller or a child coordinator representing the route.
    /// - Parameter route: the route for which to request the new view controller or child coordinator.
    /// - Returns: a result of creating a child for the route.
    // swiftlint:disable:next function_body_length
    private func makeChild(for route: SettingsNavigationRoute) -> MakeChildResult {
        switch route {
        case .root:
            let controller = SettingsViewController(
                interactor: interactorFactory.makeSettingsInteractor(),
                alertPresenter: AlertPresenter(context: self)
            )
            controller.delegate = self
            return .viewController(controller)

        case .vpnSettings:
            return .childCoordinator(VPNSettingsCoordinator(
                navigationController: navigationController,
                interactorFactory: interactorFactory,
                ipOverrideRepository: ipOverrideRepository
            ))

        case .problemReport:
            return .viewController(ProblemReportViewController(
                interactor: interactorFactory.makeProblemReportInteractor(),
                alertPresenter: AlertPresenter(context: self)
            ))

        case .apiAccess:
            return .childCoordinator(ListAccessMethodCoordinator(
                navigationController: navigationController,
                accessMethodRepository: accessMethodRepository,
                proxyConfigurationTester: proxyConfigurationTester
            ))

        case .faq:
            // Handled separately and presented as a modal.
            return .failed

        case .multihop:
            let viewModel = MultihopTunnelSettingsViewModel(tunnelManager: interactorFactory.tunnelManager)
            let view = SettingsMultihopView(tunnelViewModel: viewModel)

            let host = UIHostingController(rootView: view)
            host.title = NSLocalizedString(
                "NAVIGATION_TITLE_MULTIHOP",
                tableName: "Settings",
                value: "Multihop",
                comment: ""
            )

            return .viewController(host)

        case .daita:
            let viewModel = DAITATunnelSettingsViewModel(tunnelManager: interactorFactory.tunnelManager)
            let view = SettingsDAITAView(tunnelViewModel: viewModel)

            viewModel.didFailDAITAValidation = { [weak self] result in
                self?.showPrompt(
                    for: result.item,
                    onSave: {
                        viewModel.value = result.setting
                    },
                    onDiscard: {}
                )
            }

            let host = UIHostingController(rootView: view)
            host.title = NSLocalizedString(
                "NAVIGATION_TITLE_DAITA",
                tableName: "Settings",
                value: "DAITA",
                comment: ""
            )

            return .viewController(host)
        }
    }

    private func showPrompt(
        for item: DAITASettingsPromptItem,
        onSave: @escaping () -> Void,
        onDiscard: @escaping () -> Void
    ) {
        let presentation = AlertPresentation(
            id: "settings-daita-prompt",
            accessibilityIdentifier: .daitaPromptAlert,
            icon: .info,
            message: NSLocalizedString(
                "SETTINGS_DAITA_ENABLE_TEXT",
                tableName: "DAITA",
                value: item.description,
                comment: ""
            ),
            buttons: [
                AlertAction(
                    title: String(format: NSLocalizedString(
                        "SETTINGS_DAITA_ENABLE_OK_ACTION",
                        tableName: "DAITA",
                        value: "Enable %@",
                        comment: ""
                    ), item.title),
                    style: .default,
                    accessibilityId: .daitaConfirmAlertEnableButton,
                    handler: { onSave() }
                ),
                AlertAction(
                    title: NSLocalizedString(
                        "SETTINGS_DAITA_ENABLE_CANCEL_ACTION",
                        tableName: "DAITA",
                        value: "Back",
                        comment: ""
                    ),
                    style: .default,
                    handler: { onDiscard() }
                ),
            ]
        )

        AlertPresenter(context: self).showAlert(presentation: presentation, animated: true)
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
