//
//  ApplicationCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 13/01/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadTypes
import RelayCache
import UIKit

/**
 Preferred content size for controllers presented using formsheet modal presentation style.
 */
private let preferredFormSheetContentSize = CGSize(width: 480, height: 640)

/**
 Application coordinator managing split view and two navigation contexts.
 */
final class ApplicationCoordinator: Coordinator, Presenting, RootContainerViewControllerDelegate,
    UISplitViewControllerDelegate, ApplicationRouterDelegate
{
    /**
     Application router.
     */
    private var router: ApplicationRouter!

    /**
     Primary navigation container.

     On iPhone, it is used as a container for horizontal flows (TOS, Login, Revoked, Out-of-time).

     On iPad, it is used as a container to hold split view controller. Secondary navigation
     container presented modally is used for horizontal flows.
     */
    private let primaryNavigationContainer = RootContainerViewController()

    /**
     Secondary navigation container.

     On iPad, it is used in place of primary container for horizontal flows and displayed modally
     above primary container. Unused on iPhone.
     */
    private let secondaryNavigationContainer = RootContainerViewController()

    private lazy var secondaryRootConfiguration = ModalPresentationConfiguration(
        preferredContentSize: preferredFormSheetContentSize,
        modalPresentationStyle: .custom,
        isModalInPresentation: true,
        transitioningDelegate: SecondaryContextTransitioningDelegate()
    )

    private let splitViewController: CustomSplitViewController = {
        let svc = CustomSplitViewController()
        svc.minimumPrimaryColumnWidth = UIMetrics.minimumSplitViewSidebarWidth
        svc.preferredPrimaryColumnWidthFraction = UIMetrics.maximumSplitViewSidebarWidthFraction
        svc.dividerColor = UIColor.MainSplitView.dividerColor
        svc.primaryEdge = .trailing
        return svc
    }()

    private var splitTunnelCoordinator: TunnelCoordinator?
    private var splitLocationCoordinator: SelectLocationCoordinator?

    private let tunnelManager: TunnelManager
    private let storePaymentManager: StorePaymentManager
    private let relayCacheTracker: RelayCacheTracker

    private let apiProxy: REST.APIProxy
    private let devicesProxy: REST.DevicesProxy
    private var tunnelObserver: TunnelObserver?

    private var outOfTimeTimer: Timer?

    var rootViewController: UIViewController {
        return primaryNavigationContainer
    }

    init(
        tunnelManager: TunnelManager,
        storePaymentManager: StorePaymentManager,
        relayCacheTracker: RelayCacheTracker,
        apiProxy: REST.APIProxy,
        devicesProxy: REST.DevicesProxy
    ) {
        self.tunnelManager = tunnelManager
        self.storePaymentManager = storePaymentManager
        self.relayCacheTracker = relayCacheTracker
        self.apiProxy = apiProxy
        self.devicesProxy = devicesProxy

        /*
          Uncomment if you'd like to test TOS again
         TermsOfService.unsetAgreed()
          */

        super.init()

        primaryNavigationContainer.delegate = self
        secondaryNavigationContainer.delegate = self

        router = ApplicationRouter(self)

        addTunnelObserver()
    }

    func start() {
        if isPad {
            setupSplitView()
        }

        continueFlow(animated: false)
    }

    // MARK: - ApplicationRouterDelegate

    func applicationRouter(
        _ router: ApplicationRouter,
        route: AppRoute,
        animated: Bool,
        completion: @escaping (Coordinator) -> Void
    ) {
        switch route {
        case let .settings(subRoute):
            presentSettings(route: subRoute, animated: animated, completion: completion)

        case .selectLocation:
            presentSelectLocation(animated: animated, completion: completion)

        case .outOfTime:
            presentOutOfTime(animated: animated, completion: completion)

        case .revoked:
            presentRevoked(animated: animated, completion: completion)

        case .login:
            presentLogin(animated: animated, completion: completion)

        case .tos:
            presentTOS(animated: animated, completion: completion)

        case .main:
            presentMain(animated: animated, completion: completion)
        }
    }

    func applicationRouter(
        _ router: ApplicationRouter,
        dismissWithContext context: RouteDismissalContext,
        completion: @escaping () -> Void
    ) {
        if context.isClosing {
            let dismissedRoute = context.dismissedRoutes.first!

            switch dismissedRoute.route.routeGroup {
            case .primary:
                endHorizontalFlow(animated: context.isAnimated, completion: completion)
                context.dismissedRoutes.forEach { $0.coordinator.removeFromParent() }

            case .selectLocation, .settings:
                let coordinator = dismissedRoute.coordinator as! Presentable

                coordinator.dismiss(animated: context.isAnimated, completion: completion)
            }
        } else {
            let dismissedRoute = context.dismissedRoutes.first!
            assert(context.dismissedRoutes.count == 1)

            if case .outOfTime = dismissedRoute.route {
                let coordinator = dismissedRoute.coordinator as! OutOfTimeCoordinator

                coordinator.popFromNavigationStack(
                    animated: context.isAnimated,
                    completion: completion
                )

                coordinator.removeFromParent()
            } else {
                assertionFailure("Unhandled dismissal for \(dismissedRoute.route)")
                completion()
            }
        }
    }

    func applicationRouter(_ router: ApplicationRouter, shouldPresent route: AppRoute) -> Bool {
        switch route {
        case .revoked:
            // Check if device is still revoked.
            return tunnelManager.deviceState == .revoked

        case .outOfTime:
            // Check if device is still out of time.
            return tunnelManager.deviceState.accountData?.isExpired ?? false

        default:
            return true
        }
    }

    func applicationRouter(
        _ router: ApplicationRouter,
        shouldDismissWithContext context: RouteDismissalContext
    ) -> Bool {
        return context.dismissedRoutes.allSatisfy { dismissedRoute in
            /*
             Prevent dismissal of "out of time" route in response to device state change when
             making payment. It will dismiss itself once done.
             */
            if dismissedRoute.route == .outOfTime {
                let coordinator = dismissedRoute.coordinator as! OutOfTimeCoordinator

                return !coordinator.isMakingPayment
            }

            return true
        }
    }

    func applicationRouter(
        _ router: ApplicationRouter,
        handleSubNavigationWithContext context: RouteSubnavigationContext,
        completion: @escaping () -> Void
    ) {
        switch context.route {
        case let .settings(subRoute):
            let coordinator = context.presentedRoute.coordinator as! SettingsCoordinator

            if let subRoute = subRoute {
                coordinator.navigate(
                    to: subRoute,
                    animated: context.isAnimated,
                    completion: completion
                )
            } else {
                completion()
            }

        default:
            completion()
        }
    }

    // MARK: - Private

    /**
     Continues application flow by evaluating what route to present next.
     */
    private func continueFlow(animated: Bool) {
        let next = evaluateNextRoute()

        /*
         On iPad the main route is always visible as it's a part of root controller hence we never
         ask router to navigate to it. Instead this is when we hide the primary horizontal
         navigation.
         */
        if isPad, next == .main {
            router.dismissAll(.primary, animated: animated)
        } else {
            router.present(next, animated: animated)
        }
    }

    private func evaluateNextRoute() -> AppRoute {
        guard TermsOfService.isAgreed else {
            return .tos
        }

        switch tunnelManager.deviceState {
        case .revoked:
            return .revoked

        case .loggedOut:
            return .login

        case let .loggedIn(accountData, _):
            return accountData.isExpired ? .outOfTime : .main
        }
    }

    private func logoutRevokedDevice() {
        tunnelManager.unsetAccount { [weak self] in
            self?.continueFlow(animated: true)
        }
    }

    private func didDismissSettings(_ reason: SettingsDismissReason) {
        if isPad {
            router.dismissAll(.settings, animated: true)

            if reason == .userLoggedOut {
                router.dismissAll(.primary, animated: true)
                continueFlow(animated: true)
            }
        } else {
            if reason == .userLoggedOut {
                router.dismissAll(.primary, animated: false)
                continueFlow(animated: false)
            }

            router.dismissAll(.settings, animated: true)
        }
    }

    /**
     Navigation controller used for horizontal flows.
     */
    private var horizontalFlowController: RootContainerViewController {
        if isPad {
            return secondaryNavigationContainer
        } else {
            return primaryNavigationContainer
        }
    }

    /**
     Begins horizontal flow presenting a navigation controller suitable for current user interface
     idiom.

     On iPad this takes care of presenting a secondary navigation context using modal presentation
     after calling the given `block`.

     On iPhone this function simply passes the primary navigation container to the `block` and
     nothing else.
     */
    private func beginHorizontalFlow(animated: Bool, completion: @escaping () -> Void) {
        if isPad, secondaryNavigationContainer.presentingViewController == nil {
            secondaryRootConfiguration.apply(to: secondaryNavigationContainer)

            primaryNavigationContainer.present(
                secondaryNavigationContainer,
                animated: animated,
                completion: completion
            )
        } else {
            completion()
        }
    }

    /**
     Marks the end of horizontal flow.

     On iPad this method dismisses the modally presented  secondary navigation container and
     releases all child view controllers from it.

     Does nothing on iPhone.
     */
    private func endHorizontalFlow(animated: Bool = true, completion: (() -> Void)? = nil) {
        if isPad {
            secondaryNavigationContainer.dismiss(animated: animated, completion: completion)
        } else {
            completion?()
        }
    }

    private var isPad: Bool {
        return UIDevice.current.userInterfaceIdiom == .pad
    }

    private func setupSplitView() {
        let tunnelCoordinator = makeTunnelCoordinator()
        let selectLocationCoordinator = makeSelectLocationCoordinator(forModalPresentation: false)

        addChild(tunnelCoordinator)
        addChild(selectLocationCoordinator)

        splitTunnelCoordinator = tunnelCoordinator
        splitLocationCoordinator = selectLocationCoordinator

        splitViewController.delegate = self
        splitViewController.viewControllers = [
            selectLocationCoordinator.navigationController,
            tunnelCoordinator.rootViewController,
        ]

        primaryNavigationContainer.setViewControllers([splitViewController], animated: false)

        tunnelCoordinator.start()
        selectLocationCoordinator.start()
    }

    private func presentTOS(animated: Bool, completion: @escaping (Coordinator) -> Void) {
        let coordinator = TermsOfServiceCoordinator(navigationController: horizontalFlowController)

        coordinator.didFinish = { [weak self] coordinator in
            self?.continueFlow(animated: true)
        }

        addChild(coordinator)
        coordinator.start()

        beginHorizontalFlow(animated: animated) {
            completion(coordinator)
        }
    }

    private func presentMain(animated: Bool, completion: @escaping (Coordinator) -> Void) {
        precondition(!isPad)

        let tunnelCoordinator = makeTunnelCoordinator()

        horizontalFlowController.pushViewController(
            tunnelCoordinator.rootViewController,
            animated: animated
        )

        addChild(tunnelCoordinator)
        tunnelCoordinator.start()

        beginHorizontalFlow(animated: animated) {
            completion(tunnelCoordinator)
        }
    }

    private func presentRevoked(animated: Bool, completion: @escaping (Coordinator) -> Void) {
        let coordinator = RevokedCoordinator(
            navigationController: horizontalFlowController,
            tunnelManager: tunnelManager
        )

        coordinator.didFinish = { [weak self] coordinator in
            self?.logoutRevokedDevice()
        }

        addChild(coordinator)
        coordinator.start(animated: animated)

        beginHorizontalFlow(animated: animated) {
            completion(coordinator)
        }
    }

    private func presentOutOfTime(animated: Bool, completion: @escaping (Coordinator) -> Void) {
        let coordinator = OutOfTimeCoordinator(
            navigationController: horizontalFlowController,
            storePaymentManager: storePaymentManager,
            tunnelManager: tunnelManager
        )

        coordinator.didFinishPayment = { [weak self] coordinator in
            guard let self = self else { return }

            if self.shouldDismissOutOfTime() {
                self.router.dismiss(.outOfTime, animated: true)

                self.continueFlow(animated: true)
            }
        }

        addChild(coordinator)
        coordinator.start(animated: animated)

        beginHorizontalFlow(animated: animated) {
            completion(coordinator)
        }
    }

    private func shouldDismissOutOfTime() -> Bool {
        return !(tunnelManager.deviceState.accountData?.isExpired ?? false)
    }

    private func presentSelectLocation(
        animated: Bool,
        completion: @escaping (Coordinator) -> Void
    ) {
        let coordinator = makeSelectLocationCoordinator(forModalPresentation: true)
        coordinator.start()

        presentChild(coordinator, animated: animated) {
            completion(coordinator)
        }
    }

    private func presentLogin(animated: Bool, completion: @escaping (Coordinator) -> Void) {
        let coordinator = LoginCoordinator(
            navigationController: horizontalFlowController,
            tunnelManager: tunnelManager,
            devicesProxy: devicesProxy
        )

        coordinator.didFinish = { [weak self] coordinator in
            self?.continueFlow(animated: true)
        }

        addChild(coordinator)
        coordinator.start(animated: animated)

        beginHorizontalFlow(animated: animated) {
            completion(coordinator)
        }
    }

    private func makeTunnelCoordinator() -> TunnelCoordinator {
        let tunnelCoordinator = TunnelCoordinator(tunnelManager: tunnelManager)

        tunnelCoordinator.showSelectLocationPicker = { [weak self] in
            self?.router.present(.selectLocation, animated: true)
        }

        return tunnelCoordinator
    }

    private func makeSelectLocationCoordinator(forModalPresentation isModalPresentation: Bool)
        -> SelectLocationCoordinator
    {
        let navigationController = CustomNavigationController()
        navigationController.isNavigationBarHidden = !isModalPresentation

        let selectLocationCoordinator = SelectLocationCoordinator(
            navigationController: navigationController,
            tunnelManager: tunnelManager,
            relayCacheTracker: relayCacheTracker
        )

        selectLocationCoordinator.didFinish = { [weak self] coordinator, relay in
            if isModalPresentation {
                self?.router.dismiss(.selectLocation, animated: true)
            }
        }

        return selectLocationCoordinator
    }

    private func presentSettings(
        route: SettingsNavigationRoute?,
        animated: Bool,
        completion: @escaping (Coordinator) -> Void
    ) {
        let interactorFactory = SettingsInteractorFactory(
            storePaymentManager: storePaymentManager,
            tunnelManager: tunnelManager,
            apiProxy: apiProxy
        )

        let navigationController = CustomNavigationController()
        let coordinator = SettingsCoordinator(
            navigationController: navigationController,
            interactorFactory: interactorFactory
        )

        coordinator.didFinish = { [weak self] coordinator, reason in
            self?.didDismissSettings(reason)
        }

        coordinator.willNavigate = { [weak self] coordinator, from, to in
            if to == .root {
                self?.onShowSettings?()
            }
        }

        coordinator.navigate(to: route ?? .root, animated: false)

        coordinator.start()

        presentChild(
            coordinator,
            animated: animated,
            configuration: ModalPresentationConfiguration(
                preferredContentSize: preferredFormSheetContentSize,
                modalPresentationStyle: .formSheet
            )
        ) {
            completion(coordinator)
        }
    }

    private func addTunnelObserver() {
        let tunnelObserver =
            TunnelBlockObserver(didUpdateDeviceState: { [weak self] manager, deviceState in
                self?.deviceStateDidChange(deviceState)
            })

        tunnelManager.addObserver(tunnelObserver)

        self.tunnelObserver = tunnelObserver

        splitViewController.preferredDisplayMode = tunnelManager.deviceState.splitViewMode
    }

    private func deviceStateDidChange(_ deviceState: DeviceState) {
        splitViewController.preferredDisplayMode = deviceState.splitViewMode

        switch deviceState {
        case let .loggedIn(accountData, _):
            updateOutOfTimeTimer()

            if !accountData.isExpired {
                router.dismiss(.outOfTime, animated: true)
            }

        case .revoked:
            cancelOutOfTimeTimer()
            router.present(.revoked, animated: true)

        case .loggedOut:
            cancelOutOfTimeTimer()
        }
    }

    // MARK: - Out of time

    private func updateOutOfTimeTimer() {
        cancelOutOfTimeTimer()

        guard let expiry = tunnelManager.deviceState.accountData?.expiry else { return }

        let timer = Timer(fire: expiry, interval: 0, repeats: false, block: { [weak self] _ in
            self?.outOfTimeTimerDidFire()
        })

        RunLoop.main.add(timer, forMode: .common)

        outOfTimeTimer = timer
    }

    private func outOfTimeTimerDidFire() {
        router.present(.outOfTime, animated: true)
    }

    private func cancelOutOfTimeTimer() {
        outOfTimeTimer?.invalidate()
        outOfTimeTimer = nil
    }

    // MARK: - Settings

    var onShowSettings: (() -> Void)?

    var isPresentingSettings: Bool {
        return router.isPresenting(.settings)
    }

    // MARK: - Deep link

    func showAccountSettings() {
        router.present(.settings(.account))
    }

    // MARK: - UISplitViewControllerDelegate

    func primaryViewController(forExpanding splitViewController: UISplitViewController)
        -> UIViewController?
    {
        return splitLocationCoordinator?.navigationController
    }

    func primaryViewController(forCollapsing splitViewController: UISplitViewController)
        -> UIViewController?
    {
        return splitTunnelCoordinator?.rootViewController
    }

    func splitViewController(
        _ splitViewController: UISplitViewController,
        collapseSecondary secondaryViewController: UIViewController,
        onto primaryViewController: UIViewController
    ) -> Bool {
        return true
    }

    func splitViewController(
        _ splitViewController: UISplitViewController,
        separateSecondaryFrom primaryViewController: UIViewController
    ) -> UIViewController? {
        return nil
    }

    func splitViewControllerDidExpand(_ svc: UISplitViewController) {
        router.dismissAll(.selectLocation, animated: false)
    }

    // MARK: - RootContainerViewControllerDelegate

    func rootContainerViewControllerShouldShowSettings(
        _ controller: RootContainerViewController,
        navigateTo route: SettingsNavigationRoute?,
        animated: Bool
    ) {
        router.present(.settings(route), animated: animated)
    }

    func rootContainerViewSupportedInterfaceOrientations(_ controller: RootContainerViewController)
        -> UIInterfaceOrientationMask
    {
        if isPad {
            return [.landscape, .portrait]
        } else {
            return [.portrait]
        }
    }

    func rootContainerViewAccessibilityPerformMagicTap(_ controller: RootContainerViewController)
        -> Bool
    {
        guard tunnelManager.deviceState.isLoggedIn else { return false }

        switch tunnelManager.tunnelStatus.state {
        case .connected, .connecting, .reconnecting, .waitingForConnectivity:
            tunnelManager.reconnectTunnel(selectNewRelay: true)

        case .disconnecting, .disconnected:
            tunnelManager.startTunnel()

        case .pendingReconnect:
            break
        }

        return true
    }

    // MARK: - Presenting

    var presentationContext: UIViewController {
        return primaryNavigationContainer.presentedViewController ?? primaryNavigationContainer
    }
}

extension DeviceState {
    var splitViewMode: UISplitViewController.DisplayMode {
        return isLoggedIn ? .allVisible : .secondaryOnly
    }
}
