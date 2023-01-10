//
//  SettingsNavigationCoordinator.swift
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

    var isModalPresentation: Bool {
        return self == .faq
    }
}

enum SettingsDismissReason: Equatable {
    case none
    case userLoggedOut
}

protocol SettingsNavigationCoordinatorDelegate: AnyObject {
    func settingsNavigationCoordinator(
        _ coordinator: SettingsNavigationCoordinator,
        willNavigateFrom source: SettingsNavigationRoute?,
        to destination: SettingsNavigationRoute
    )

    func settingsNavigationCoordinator(
        _ coordinator: SettingsNavigationCoordinator,
        didFinishWithReason reason: SettingsDismissReason
    )
}

final class SettingsNavigationCoordinator: NSObject, UIAdaptivePresentationControllerDelegate,
    SettingsViewControllerDelegate, AccountViewControllerDelegate, SFSafariViewControllerDelegate
{
    private var navigator: Navigator?

    private let interactorFactory: SettingsInteractorFactory
    private var currentRoute: SettingsNavigationRoute?

    private let transitionQueue = AsyncOperationQueue.makeSerial()
    private let logger = Logger(label: "SettingsNavigationCoordinator")

    weak var delegate: SettingsNavigationCoordinatorDelegate?

    init(interactorFactory: SettingsInteractorFactory) {
        self.interactorFactory = interactorFactory

        super.init()
    }

    // MARK: - Presentation

    var isPresented: Bool {
        return navigator?.navigationController.isPresented ?? false
    }

    func present(
        route: SettingsNavigationRoute?,
        from parent: UIViewController,
        animated: Bool,
        completion: (() -> Void)? = nil
    ) {
        enqueueTransition { finish in
            self.presentNoQueue(route: route, from: parent, animated: animated) {
                completion?()
                finish()
            }
        }
    }

    func dismiss(animated: Bool, completion: (() -> Void)? = nil) {
        enqueueTransition { finish in
            self.dismissNoQueue(animated: animated) {
                completion?()
                finish()
            }
        }
    }

    // MARK: - Navigation

    private func show(
        route: SettingsNavigationRoute,
        animated: Bool,
        completion: (() -> Void)? = nil
    ) {
        enqueueTransition { finish in
            self.showNoQueue(route: route, animated: animated) {
                completion?()
                finish()
            }
        }
    }

    private func showNoQueue(
        route: SettingsNavigationRoute,
        animated: Bool,
        completion: (() -> Void)? = nil
    ) {
        guard let navigator = navigator else {
            completion?()
            return
        }

        showNoQueueInner(
            navigator: navigator,
            route: route,
            animated: animated,
            completion: completion
        )
    }

    private func showNoQueueInner(
        navigator: Navigator,
        route: SettingsNavigationRoute,
        animated: Bool,
        completion: (() -> Void)? = nil
    ) {
        if route.isModalPresentation {
            navigator.navigationController.present(
                makeViewController(for: route),
                animated: animated,
                completion: completion
            )
        } else {
            pushInline(
                navigator: navigator,
                route: route,
                animated: animated,
                completion: completion
            )
        }
    }

    private func presentNoQueue(
        route: SettingsNavigationRoute?,
        from parent: UIViewController,
        animated: Bool,
        completion: @escaping () -> Void
    ) {
        let isModalRoute = route?.isModalPresentation ?? false

        if let navigator = navigator, let route = route {
            showNoQueueInner(
                navigator: navigator,
                route: route,
                animated: animated,
                completion: completion
            )
        } else if navigator == nil, !isModalRoute {
            let navigator = makeNavigator()
            self.navigator = navigator

            if let route = route {
                pushInline(navigator: navigator, route: route, animated: false)
            }

            presentNavigator(navigator, from: parent, animated: animated, completion: completion)
        } else {
            completion()
        }
    }

    private func dismissNoQueue(animated: Bool, completion: (() -> Void)? = nil) {
        guard let navigator = navigator else {
            completion?()
            return
        }

        navigator.navigationController.dismiss(animated: animated) {
            self.navigatorDidDismiss()
            completion?()
        }
    }

    private func pushInline(
        navigator: Navigator,
        route: SettingsNavigationRoute,
        animated: Bool,
        completion: (() -> Void)? = nil
    ) {
        switch route {
        case .root:
            navigator.popToRoot(animated: animated, completion: completion)

        default:
            let nextViewController = makeViewController(for: route)
            let viewControllers = navigator.children

            if let rootController = viewControllers.first, viewControllers.count > 1 {
                navigator.replace(
                    [rootController, nextViewController],
                    animated: animated,
                    completion: completion
                )
            } else {
                navigator.push(nextViewController, animated: animated, completion: completion)
            }
        }
    }

    private func presentNavigator(
        _ navigator: Navigator,
        from parent: UIViewController,
        animated: Bool,
        completion: @escaping () -> Void
    ) {
        let navigationController = navigator.navigationController

        guard !navigationController.isPresented else {
            completion()
            return
        }

        if UIDevice.current.userInterfaceIdiom == .pad {
            navigationController.preferredContentSize = CGSize(width: 480, height: 568)
            navigationController.modalPresentationStyle = .formSheet
        }

        navigationController.presentationController?.delegate = self

        parent.present(navigationController, animated: animated, completion: completion)
    }

    private func makeNavigator() -> Navigator {
        let navigator = Navigator(navigationController: SettingsNavigationController())
        navigator.push(makeViewController(for: .root), animated: false)
        navigator.willShow = { [weak self] vc in
            guard let self = self, let route = self.route(for: vc) else { return }

            self.logger.debug(
                "Navigate from \(self.currentRoute.map { "\($0)" } ?? "none") -> \(route)"
            )

            self.delegate?.settingsNavigationCoordinator(
                self,
                willNavigateFrom: self.currentRoute,
                to: route
            )

            self.currentRoute = route
        }
        return navigator
    }

    private func navigatorDidDismiss() {
        navigator = nil
        currentRoute = nil
    }

    private func enqueueTransition(_ body: @escaping (_ finish: @escaping () -> Void) -> Void) {
        let operation = AsyncBlockOperation(dispatchQueue: .main) { operation in
            body {
                operation.finish()
            }
        }
        transitionQueue.addOperation(operation)
    }

    // MARK: - UIAdaptivePresentationControllerDelegate

    func presentationControllerDidDismiss(_ presentationController: UIPresentationController) {
        navigatorDidDismiss()
        delegate?.settingsNavigationCoordinator(self, didFinishWithReason: .none)
    }

    // MARK: - SettingsViewControllerDelegate

    func settingsViewControllerDidFinish(_ controller: SettingsViewController) {
        delegate?.settingsNavigationCoordinator(self, didFinishWithReason: .none)
    }

    func settingsViewController(
        _ controller: SettingsViewController,
        didRequestRoutePresentation route: SettingsNavigationRoute
    ) {
        show(route: route, animated: true)
    }

    // MARK: - AccountViewControllerDelegate

    func accountViewControllerDidLogout(_ controller: AccountViewController) {
        delegate?.settingsNavigationCoordinator(self, didFinishWithReason: .userLoggedOut)
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
            let safariViewController = SFSafariViewController(
                url: ApplicationConfiguration.faqAndGuidesURL
            )
            safariViewController.delegate = self

            return safariViewController
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

    // MARK: - SFSafariViewControllerDelegate

    func safariViewControllerDidFinish(_ controller: SFSafariViewController) {
        enqueueTransition { finish in
            controller.dismiss(animated: true, completion: finish)
        }
    }

    func safariViewControllerWillOpenInBrowser(_ controller: SFSafariViewController) {
        enqueueTransition { finish in
            controller.dismiss(animated: false, completion: finish)
        }
    }
}

extension UIViewController {
    var isPresented: Bool {
        return presentingViewController != nil
    }
}
