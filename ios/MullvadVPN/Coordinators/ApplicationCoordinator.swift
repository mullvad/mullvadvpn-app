//
//  ApplicationCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 13/01/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadREST
import MullvadSettings
import MullvadTypes
import Routing
import UIKit

/**
 Application coordinator managing split view and two navigation contexts.
 */
final class ApplicationCoordinator: Coordinator, Presenting, @preconcurrency RootContainerViewControllerDelegate,
    UISplitViewControllerDelegate, @preconcurrency ApplicationRouterDelegate,
    @preconcurrency NotificationManagerDelegate {
    typealias RouteType = AppRoute

    /**
     Application router.
     */
    nonisolated(unsafe) private(set) var router: ApplicationRouter<AppRoute>!

    /**
     Navigation container.

     Used as a container for horizontal flows (TOS, Login, Revoked, Out-of-time).
     */
    private let navigationContainer = RootContainerViewController()

    /// Posts `preferredAccountNumber` notification when user inputs the account number instead of voucher code
    private let preferredAccountNumberSubject = PassthroughSubject<String, Never>()

    private let notificationController = NotificationController()

    private var splitTunnelCoordinator: TunnelCoordinator?
    private var splitLocationCoordinator: LocationCoordinator?

    private let tunnelManager: TunnelManager
    private let storePaymentManager: StorePaymentManager
    private let relayCacheTracker: RelayCacheTracker

    private let apiProxy: APIQuerying
    private let devicesProxy: DeviceHandling
    private let accountsProxy: RESTAccountHandling
    private var tunnelObserver: TunnelObserver?
    private var appPreferences: AppPreferencesDataSource
    private var outgoingConnectionService: OutgoingConnectionServiceHandling
    private var accessMethodRepository: AccessMethodRepositoryProtocol
    private let configuredTransportProvider: ProxyConfigurationTransportProvider
    private let ipOverrideRepository: IPOverrideRepository
    private let relaySelectorWrapper: RelaySelectorWrapper

    private var outOfTimeTimer: Timer?

    var rootViewController: UIViewController {
        navigationContainer
    }

    init(
        tunnelManager: TunnelManager,
        storePaymentManager: StorePaymentManager,
        relayCacheTracker: RelayCacheTracker,
        apiProxy: APIQuerying,
        devicesProxy: DeviceHandling,
        accountsProxy: RESTAccountHandling,
        outgoingConnectionService: OutgoingConnectionServiceHandling,
        appPreferences: AppPreferencesDataSource,
        accessMethodRepository: AccessMethodRepositoryProtocol,
        transportProvider: ProxyConfigurationTransportProvider,
        ipOverrideRepository: IPOverrideRepository,
        relaySelectorWrapper: RelaySelectorWrapper

    ) {
        self.tunnelManager = tunnelManager
        self.storePaymentManager = storePaymentManager
        self.relayCacheTracker = relayCacheTracker
        self.apiProxy = apiProxy
        self.devicesProxy = devicesProxy
        self.accountsProxy = accountsProxy
        self.appPreferences = appPreferences
        self.outgoingConnectionService = outgoingConnectionService
        self.accessMethodRepository = accessMethodRepository
        self.configuredTransportProvider = transportProvider
        self.ipOverrideRepository = ipOverrideRepository
        self.relaySelectorWrapper = relaySelectorWrapper

        super.init()

        navigationContainer.delegate = self

        router = ApplicationRouter(self)

        addTunnelObserver()

        NotificationManager.shared.delegate = self
    }

    func start() {
        navigationContainer.notificationController = notificationController

        continueFlow(animated: false)
    }

    // MARK: - ApplicationRouterDelegate

    func applicationRouter(
        _ router: ApplicationRouter<RouteType>,
        presentWithContext context: RoutePresentationContext<RouteType>,
        animated: Bool,
        completion: @escaping @Sendable (Coordinator) -> Void
    ) {
        switch context.route {
        case .account:
            presentAccount(animated: animated, completion: completion)

        case let .settings(subRoute):
            presentSettings(route: subRoute, animated: animated, completion: completion)

        case .daita:
            presentDAITA(animated: animated, completion: completion)

        case .selectLocation:
            presentSelectLocation(animated: animated, completion: completion)

        case .outOfTime:
            presentOutOfTime(animated: animated, completion: completion)

        case .revoked:
            presentRevoked(animated: animated, completion: completion)

        case .login:
            presentLogin(animated: animated, completion: completion)

        case .changelog:
            presentChangeLog(animated: animated, completion: completion)

        case .tos:
            presentTOS(animated: animated, completion: completion)

        case .main:
            presentMain(animated: animated, completion: completion)

        case .welcome:
            presentWelcome(animated: animated, completion: completion)

        case .alert:
            presentAlert(animated: animated, context: context, completion: completion)
        }
    }

    func applicationRouter(
        _ router: ApplicationRouter<RouteType>,
        dismissWithContext context: RouteDismissalContext<RouteType>,
        completion: @escaping @Sendable () -> Void
    ) {
        let dismissedRoute = context.dismissedRoutes.first!

        if context.isClosing {
            switch dismissedRoute.route.routeGroup {
            case .primary:
                completion()
                context.dismissedRoutes.forEach { $0.coordinator.removeFromParent() }

            case .selectLocation, .account, .settings, .changelog, .alert:
                guard let coordinator = dismissedRoute.coordinator as? Presentable else {
                    completion()
                    return assertionFailure("Expected presentable coordinator for \(dismissedRoute.route)")
                }

                coordinator.dismiss(animated: context.isAnimated, completion: completion)
            }
        } else {
            assert(context.dismissedRoutes.count == 1)

            switch dismissedRoute.route {
            case .outOfTime, .welcome:
                guard let coordinator = dismissedRoute.coordinator as? Poppable else {
                    completion()
                    return assertionFailure("Expected presentable coordinator for \(dismissedRoute.route)")
                }

                coordinator.popFromNavigationStack(
                    animated: context.isAnimated,
                    completion: completion
                )

                coordinator.removeFromParent()

            default:
                assertionFailure("Unhandled dismissal for \(dismissedRoute.route)")
                completion()
            }
        }
    }

    func applicationRouter(_ router: ApplicationRouter<RouteType>, shouldPresent route: RouteType) -> Bool {
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
        _ router: ApplicationRouter<RouteType>,
        shouldDismissWithContext context: RouteDismissalContext<RouteType>
    ) -> Bool {
        context.dismissedRoutes.allSatisfy { dismissedRoute in
            /*
             Prevent dismissal of "out of time" route in response to device state change when
             making payment. It will dismiss itself once done.
             */
            if dismissedRoute.route == .outOfTime {
                guard let coordinator = dismissedRoute.coordinator as? OutOfTimeCoordinator else {
                    return false
                }
                return !coordinator.isMakingPayment
            }

            return true
        }
    }

    func applicationRouter(
        _ router: ApplicationRouter<RouteType>,
        handleSubNavigationWithContext context: RouteSubnavigationContext<RouteType>,
        completion: @escaping @Sendable @MainActor () -> Void
    ) {
        switch context.route {
        case let .settings(subRoute):
            guard let coordinator = context.presentedRoute.coordinator as? SettingsCoordinator else { return }
            if let subRoute {
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

    private var isPresentingAccountExpiryBanner = false

    /**
     Continues application flow by evaluating what route to present next.
     */
    private func continueFlow(animated: Bool) {
        let nextRoutes = evaluateNextRoutes()

        for nextRoute in nextRoutes {
            router.present(nextRoute, animated: animated)
        }
    }

    /**
     Evaluates conditions and returns the routes that need to be presented next.
     */
    private func evaluateNextRoutes() -> [AppRoute] {
        // Show TOS alone blocking all other routes.
        guard appPreferences.isAgreedToTermsOfService else {
            return [.tos]
        }

        var routes = [AppRoute]()

        // Pick the primary route to present
        switch tunnelManager.deviceState {
        case .revoked:
            routes.append(.revoked)

        case .loggedOut:
            routes.append(.login)

        case let .loggedIn(accountData, _):
            if !appPreferences.isShownOnboarding {
                routes.append(.welcome)
            } else {
                routes.append(accountData.isExpired ? .outOfTime : .main)
            }
        }

        return routes
    }

    private func logoutRevokedDevice() {
        Task { [weak self] in
            guard let self else { return }
            await tunnelManager.unsetAccount()
            continueFlow(animated: true)
        }
    }

    private func didDismissAccount(_ reason: AccountDismissReason) {
        if reason == .userLoggedOut {
            router.dismissAll(.primary, animated: false)
            continueFlow(animated: false)
        }
        router.dismiss(.account, animated: true)
    }

    private func presentTOS(animated: Bool, completion: @escaping (Coordinator) -> Void) {
        let coordinator = TermsOfServiceCoordinator(navigationController: navigationContainer)

        coordinator.didFinish = { [weak self] _ in
            self?.appPreferences.isAgreedToTermsOfService = true
            self?.continueFlow(animated: true)
        }

        addChild(coordinator)
        coordinator.start()

        completion(coordinator)
    }

    private func presentChangeLog(animated: Bool, completion: @escaping (Coordinator) -> Void) {
        let coordinator = ChangeLogCoordinator(
            route: .changelog,
            navigationController: CustomNavigationController(),
            viewModel: ChangeLogViewModel(changeLogReader: ChangeLogReader())
        )

        coordinator.didFinish = { [weak self] _ in
            self?.router.dismiss(.changelog, animated: animated)
        }

        coordinator.start(animated: false)

        presentChild(coordinator, animated: animated) {
            completion(coordinator)
        }
    }

    private func presentMain(animated: Bool, completion: @escaping (Coordinator) -> Void) {
        let tunnelCoordinator = makeTunnelCoordinator()

        navigationContainer.pushViewController(
            tunnelCoordinator.rootViewController,
            animated: animated
        )

        addChild(tunnelCoordinator)
        tunnelCoordinator.start()

        completion(tunnelCoordinator)
    }

    private func presentRevoked(animated: Bool, completion: @escaping (Coordinator) -> Void) {
        let coordinator = RevokedCoordinator(
            navigationController: navigationContainer,
            tunnelManager: tunnelManager
        )

        coordinator.didFinish = { [weak self] _ in
            self?.logoutRevokedDevice()
        }

        addChild(coordinator)
        coordinator.start(animated: animated)

        completion(coordinator)
    }

    private func presentOutOfTime(animated: Bool, completion: @escaping (Coordinator) -> Void) {
        let coordinator = OutOfTimeCoordinator(
            navigationController: navigationContainer,
            storePaymentManager: storePaymentManager,
            tunnelManager: tunnelManager
        )

        coordinator.didFinishPayment = { [weak self] _ in
            guard let self = self else { return }

            Task { @MainActor in
                if shouldDismissOutOfTime() {
                    router.dismiss(.outOfTime, animated: true)
                    continueFlow(animated: true)
                }
            }
        }

        addChild(coordinator)
        coordinator.start(animated: animated)

        completion(coordinator)
    }

    private func presentWelcome(animated: Bool, completion: @escaping (Coordinator) -> Void) {
        appPreferences.isShownOnboarding = true

        let coordinator = WelcomeCoordinator(
            navigationController: navigationContainer,
            storePaymentManager: storePaymentManager,
            tunnelManager: tunnelManager,
            accountsProxy: accountsProxy
        )
        coordinator.didFinish = { [weak self] in
            guard let self else { return }
            router.dismiss(.welcome, animated: false)
            continueFlow(animated: false)
        }
        coordinator.didLogout = { [weak self] preferredAccountNumber in
            guard let self else { return }
            router.dismissAll(.primary, animated: true)
            DispatchQueue.main.async {
                self.continueFlow(animated: true)
            }
            preferredAccountNumberSubject.send(preferredAccountNumber)
        }

        addChild(coordinator)
        coordinator.start(animated: animated)

        completion(coordinator)
    }

    private func shouldDismissOutOfTime() -> Bool {
        !(tunnelManager.deviceState.accountData?.isExpired ?? false)
    }

    private func presentSelectLocation(animated: Bool, completion: @escaping (Coordinator) -> Void) {
        let coordinator = makeLocationCoordinator(forModalPresentation: true)
        coordinator.start()

        presentChild(coordinator, animated: animated) {
            completion(coordinator)
        }
    }

    private func presentLogin(animated: Bool, completion: @escaping (Coordinator) -> Void) {
        let coordinator = LoginCoordinator(
            navigationController: navigationContainer,
            tunnelManager: tunnelManager,
            devicesProxy: devicesProxy
        )

        coordinator.preferredAccountNumberPublisher = preferredAccountNumberSubject.eraseToAnyPublisher()

        coordinator.didFinish = { [weak self] _ in
            self?.continueFlow(animated: true)
        }
        coordinator.didCreateAccount = { [weak self] in
            self?.appPreferences.isShownOnboarding = false
        }

        addChild(coordinator)
        coordinator.start(animated: animated)

        completion(coordinator)
    }

    private func presentAlert(
        animated: Bool,
        context: RoutePresentationContext<RouteType>,
        completion: @escaping (Coordinator) -> Void
    ) {
        guard let metadata = context.metadata as? AlertMetadata else {
            assertionFailure("Could not get AlertMetadata from RoutePresentationContext.")
            return
        }

        let coordinator = AlertCoordinator(presentation: metadata.presentation)

        coordinator.didFinish = { [weak self] in
            self?.router.dismiss(context.route)
        }

        coordinator.start()

        metadata.context.presentChild(coordinator, animated: animated) {
            completion(coordinator)
        }
    }

    private func makeTunnelCoordinator() -> TunnelCoordinator {
        let tunnelCoordinator = TunnelCoordinator(
            tunnelManager: tunnelManager,
            outgoingConnectionService: outgoingConnectionService,
            ipOverrideRepository: ipOverrideRepository
        )

        tunnelCoordinator.showSelectLocationPicker = { [weak self] in
            self?.router.present(.selectLocation, animated: true)
        }

        return tunnelCoordinator
    }

    private func makeLocationCoordinator(forModalPresentation isModalPresentation: Bool)
        -> LocationCoordinator {
        let navigationController = CustomNavigationController()
        navigationController.isNavigationBarHidden = !isModalPresentation

        let locationCoordinator = LocationCoordinator(
            navigationController: navigationController,
            tunnelManager: tunnelManager,
            relaySelectorWrapper: relaySelectorWrapper,
            customListRepository: CustomListRepository()
        )

        locationCoordinator.didFinish = { [weak self] _ in
            if isModalPresentation {
                self?.router.dismiss(.selectLocation, animated: true)
            }
        }

        return locationCoordinator
    }

    private func presentAccount(animated: Bool, completion: @escaping (Coordinator) -> Void) {
        let accountInteractor = AccountInteractor(
            tunnelManager: tunnelManager,
            accountsProxy: accountsProxy,
            apiProxy: apiProxy
        )

        let coordinator = AccountCoordinator(
            navigationController: CustomNavigationController(),
            interactor: accountInteractor,
            storePaymentManager: storePaymentManager
        )

        coordinator.didFinish = { [weak self] _, reason in
            self?.didDismissAccount(reason)
        }

        coordinator.start(animated: animated)

        presentChild(
            coordinator,
            animated: animated
        ) { [weak self] in
            completion(coordinator)

            self?.onShowAccount?()
        }
    }

    private func presentSettings(
        route: SettingsNavigationRoute?,
        animated: Bool,
        completion: @escaping @Sendable (Coordinator) -> Void
    ) {
        let interactorFactory = SettingsInteractorFactory(
            tunnelManager: tunnelManager,
            apiProxy: apiProxy,
            relayCacheTracker: relayCacheTracker,
            ipOverrideRepository: ipOverrideRepository
        )

        let navigationController = CustomNavigationController()
        navigationController.view.setAccessibilityIdentifier(.settingsContainerView)

        let configurationTester = ProxyConfigurationTester(transportProvider: configuredTransportProvider)

        let coordinator = SettingsCoordinator(
            navigationController: navigationController,
            interactorFactory: interactorFactory,
            accessMethodRepository: accessMethodRepository,
            proxyConfigurationTester: configurationTester,
            ipOverrideRepository: ipOverrideRepository
        )

        coordinator.didFinish = { [weak self] _ in
            Task { @MainActor in
                self?.router.dismissAll(.settings, animated: true)
            }
        }

        coordinator.willNavigate = { [weak self] _, _, to in
            if to == .root {
                self?.onShowSettings?()
            }
        }

        coordinator.start(initialRoute: route)

        presentChild(
            coordinator,
            animated: animated
        ) {
            completion(coordinator)
        }
    }

    private func presentDAITA(animated: Bool, completion: @escaping @Sendable (Coordinator) -> Void) {
        let viewModel = DAITATunnelSettingsViewModel(tunnelManager: tunnelManager)
        let coordinator = DAITASettingsCoordinator(
            navigationController: CustomNavigationController(),
            route: .daita,
            viewModel: viewModel
        )

        coordinator.didFinish = { [weak self] _ in
            self?.router.dismiss(.daita, animated: true)
        }

        coordinator.start(animated: animated)

        presentChild(coordinator, animated: animated) {
            completion(coordinator)
        }
    }

    private func addTunnelObserver() {
        let tunnelObserver =
            TunnelBlockObserver(
                didUpdateTunnelStatus: { [weak self] _, tunnelStatus in
                    if case let .error(observedState) = tunnelStatus.observedState,
                       observedState.reason == .accountExpired {
                        self?.router.present(.outOfTime)
                    }
                },
                didUpdateDeviceState: { [weak self] _, deviceState, previousDeviceState in
                    self?.deviceStateDidChange(deviceState, previousDeviceState: previousDeviceState)
                }
            )

        tunnelManager.addObserver(tunnelObserver)

        self.tunnelObserver = tunnelObserver

        updateDeviceInfo(deviceState: tunnelManager.deviceState)
    }

    private func deviceStateDidChange(_ deviceState: DeviceState, previousDeviceState: DeviceState) {
        updateDeviceInfo(deviceState: deviceState)

        switch deviceState {
        case let .loggedIn(accountData, _):

            // Account creation is being shown
            guard !isPresentingWelcome && !appPreferences.isShownOnboarding else { return }

            // Handle transition to and from expired state.
            switch (previousDeviceState.accountData?.isExpired ?? false, accountData.isExpired) {
            // add more credit
            case (true, false):
                updateOutOfTimeTimer(accountData: accountData)
                continueFlow(animated: true)
                router.dismiss(.outOfTime, animated: true)
            // account was expired
            case (false, true):
                router.present(.outOfTime, animated: true)

            default:
                break
            }
        case .revoked:
            appPreferences.isShownOnboarding = true
            cancelOutOfTimeTimer()
            router.present(.revoked, animated: true)
        case .loggedOut:
            appPreferences.isShownOnboarding = true
            cancelOutOfTimeTimer()
        }
    }

    private func updateDeviceInfo(deviceState: DeviceState) {
        let rootDeviceInfoViewModel = RootDeviceInfoViewModel(
            isPresentingAccountExpiryBanner: isPresentingAccountExpiryBanner,
            deviceState: deviceState
        )
        self.navigationContainer.update(configuration: rootDeviceInfoViewModel.configuration)
    }

    // MARK: - Out of time

    private func updateOutOfTimeTimer(accountData: StoredAccountData) {
        cancelOutOfTimeTimer()

        guard !accountData.isExpired else { return }

        let timer = Timer(fire: accountData.expiry, interval: 0, repeats: false, block: { [weak self] _ in
            Task { @MainActor in
                self?.router.present(.outOfTime, animated: true)
            }
        })

        RunLoop.main.add(timer, forMode: .common)

        outOfTimeTimer = timer
    }

    private func cancelOutOfTimeTimer() {
        outOfTimeTimer?.invalidate()
        outOfTimeTimer = nil
    }

    // MARK: - Settings

    /**
     This closure is called each time when settings are presented or when navigating from any of sub-routes within
     settings back to root.
     */
    var onShowSettings: (() -> Void)?

    /// This closure is called each time when account controller is being presented.
    var onShowAccount: (() -> Void)?

    /// Returns `true` if settings are being presented.
    var isPresentingSettings: Bool {
        router.isPresenting(group: .settings)
    }

    /// Returns `true` if account controller is being presented.
    var isPresentingAccount: Bool {
        router.isPresenting(group: .account)
    }

    /// Returns `true` if welcome controller is being presented.
    private var isPresentingWelcome: Bool {
        router.isPresenting(route: .welcome)
    }

    // MARK: - UISplitViewControllerDelegate

    func primaryViewController(forExpanding splitViewController: UISplitViewController)
        -> UIViewController? {
        splitLocationCoordinator?.navigationController
    }

    func primaryViewController(forCollapsing splitViewController: UISplitViewController)
        -> UIViewController? {
        splitTunnelCoordinator?.rootViewController
    }

    func splitViewController(
        _ splitViewController: UISplitViewController,
        collapseSecondary secondaryViewController: UIViewController,
        onto primaryViewController: UIViewController
    ) -> Bool {
        true
    }

    func splitViewController(
        _ splitViewController: UISplitViewController,
        separateSecondaryFrom primaryViewController: UIViewController
    ) -> UIViewController? {
        nil
    }

    func splitViewControllerDidExpand(_ svc: UISplitViewController) {
        router.dismissAll(.selectLocation, animated: false)
    }

    // MARK: - RootContainerViewControllerDelegate

    func rootContainerViewControllerShouldShowAccount(
        _ controller: RootContainerViewController,
        animated: Bool
    ) {
        router.present(.account, animated: animated)
    }

    func rootContainerViewControllerShouldShowSettings(
        _ controller: RootContainerViewController,
        navigateTo route: SettingsNavigationRoute?,
        animated: Bool
    ) {
        router.present(.settings(route), animated: animated)
    }

    func rootContainerViewSupportedInterfaceOrientations(_ controller: RootContainerViewController)
        -> UIInterfaceOrientationMask {
        return [.portrait]
    }

    func rootContainerViewAccessibilityPerformMagicTap(_ controller: RootContainerViewController)
        -> Bool {
        guard tunnelManager.deviceState.isLoggedIn else { return false }

        switch tunnelManager.tunnelStatus.state {
        case .connected, .connecting, .reconnecting, .waitingForConnectivity(.noConnection), .error,
             .negotiatingEphemeralPeer:
            tunnelManager.reconnectTunnel(selectNewRelay: true)

        case .disconnecting, .disconnected:
            tunnelManager.startTunnel()

        case .pendingReconnect, .waitingForConnectivity(.noNetwork):
            break
        }
        return true
    }

    // MARK: - NotificationManagerDelegate

    func notificationManagerDidUpdateInAppNotifications(
        _ manager: NotificationManager,
        notifications: [InAppNotificationDescriptor]
    ) {
        isPresentingAccountExpiryBanner = notifications
            .contains(where: { $0.identifier == .accountExpiryInAppNotification })
        updateDeviceInfo(deviceState: tunnelManager.deviceState)
        notificationController.setNotifications(notifications, animated: true)
    }

    func notificationManager(_ manager: NotificationManager, didReceiveResponse response: NotificationResponse) {
        switch response.providerIdentifier {
        case .accountExpirySystemNotification:
            router.present(.account)
        case .accountExpiryInAppNotification:
            isPresentingAccountExpiryBanner = false
            updateDeviceInfo(deviceState: tunnelManager.deviceState)
        case .latestChangesInAppNotificationProvider:
            router.present(.changelog)
        default: return
        }
    }

    // MARK: - Presenting

    var presentationContext: UIViewController {
        navigationContainer.presentedViewController ?? navigationContainer
    }
}

extension DeviceState {
    var splitViewMode: UISplitViewController.DisplayMode {
        isLoggedIn ? UISplitViewController.DisplayMode.oneBesideSecondary : .secondaryOnly
    }
} // swiftlint:disable:this file_length
