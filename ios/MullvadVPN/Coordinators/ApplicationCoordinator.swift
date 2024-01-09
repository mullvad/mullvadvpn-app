//
//  ApplicationCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 13/01/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
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
final class ApplicationCoordinator: Coordinator, Presenting, RootContainerViewControllerDelegate,
    UISplitViewControllerDelegate, ApplicationRouterDelegate, NotificationManagerDelegate {
    typealias RouteType = AppRoute

    /**
     Application router.
     */
    private(set) var router: ApplicationRouter<AppRoute>!

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

    /// Posts `preferredAccountNumber` notification when user inputs the account number instead of voucher code
    private let preferredAccountNumberSubject = PassthroughSubject<String, Never>()

    private lazy var secondaryRootConfiguration = ModalPresentationConfiguration(
        preferredContentSize: UIMetrics.preferredFormSheetContentSize,
        modalPresentationStyle: .custom,
        isModalInPresentation: true,
        transitioningDelegate: SecondaryContextTransitioningDelegate(adjustViewWhenKeyboardAppears: false)
    )

    private let notificationController = NotificationController()

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

    private let apiProxy: APIQuerying
    private let devicesProxy: DeviceHandling
    private let accountsProxy: RESTAccountHandling
    private var tunnelObserver: TunnelObserver?
    private var appPreferences: AppPreferencesDataSource
    private var outgoingConnectionService: OutgoingConnectionServiceHandling
    private var accessMethodRepository: AccessMethodRepositoryProtocol
    private let configuredTransportProvider: ConfiguredTransportProvider

    private var outOfTimeTimer: Timer?

    var rootViewController: UIViewController {
        primaryNavigationContainer
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
        transportProvider: ConfiguredTransportProvider
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

        super.init()

        primaryNavigationContainer.delegate = self
        secondaryNavigationContainer.delegate = self

        router = ApplicationRouter(self)

        addTunnelObserver()

        NotificationManager.shared.delegate = self
    }

    func start() {
        if isPad {
            setupSplitView()
        }

        setNotificationControllerParent(isPrimary: true)

        continueFlow(animated: false)
    }

    // MARK: - ApplicationRouterDelegate

    func applicationRouter(
        _ router: ApplicationRouter<RouteType>,
        presentWithContext context: RoutePresentationContext<RouteType>,
        animated: Bool,
        completion: @escaping (Coordinator) -> Void
    ) {
        switch context.route {
        case .account:
            presentAccount(animated: animated, completion: completion)

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
        completion: @escaping () -> Void
    ) {
        let dismissedRoute = context.dismissedRoutes.first!

        if context.isClosing {
            switch dismissedRoute.route.routeGroup {
            case .primary:
                endHorizontalFlow(animated: context.isAnimated, completion: completion)
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
        completion: @escaping () -> Void
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
        var nextRoutes = evaluateNextRoutes()

        if isPad {
            /*
             On iPad the main route is always visible as it's a part of root controller hence we never
             ask router to navigate to it. Instead this is when we hide the primary horizontal
             navigation.
             */
            if nextRoutes.contains(.main) {
                router.dismissAll(.primary, animated: animated)
            }

            nextRoutes.removeAll { $0 == .main }
        }

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

        // Change log can be presented simultaneously with other routes.
        if !appPreferences.hasSeenLastChanges {
            routes.append(.changelog)
        }

        return routes
    }

    private func logoutRevokedDevice() {
        tunnelManager.unsetAccount { [weak self] in
            self?.continueFlow(animated: true)
        }
    }

    private func didDismissAccount(_ reason: AccountDismissReason) {
        if isPad {
            router.dismiss(.account, animated: true)
            if reason == .userLoggedOut {
                router.dismissAll(.primary, animated: true)
                continueFlow(animated: true)
            }
        } else {
            if reason == .userLoggedOut {
                router.dismissAll(.primary, animated: false)
                continueFlow(animated: false)
            }
            router.dismiss(.account, animated: true)
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

     On iPad this takes care of presenting a secondary navigation context using modal presentation.

     On iPhone this does nothing.
     */
    private func beginHorizontalFlow(animated: Bool, completion: @escaping () -> Void) {
        if isPad, secondaryNavigationContainer.presentingViewController == nil {
            secondaryRootConfiguration.apply(to: secondaryNavigationContainer)
            addSecondaryContextPresentationStyleObserver()

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
            removeSecondaryContextPresentationStyleObserver()

            secondaryNavigationContainer.dismiss(animated: animated) {
                // Put notification controller back into primary container.
                self.setNotificationControllerParent(isPrimary: true)

                completion?()
            }
        } else {
            completion?()
        }
    }

    /**
     Assigns notification controller to either primary or secondary container making sure that only one of them holds
     the reference.
     */
    private func setNotificationControllerParent(isPrimary: Bool) {
        if isPrimary {
            secondaryNavigationContainer.notificationController = nil
            primaryNavigationContainer.notificationController = notificationController
        } else {
            primaryNavigationContainer.notificationController = nil
            secondaryNavigationContainer.notificationController = notificationController
        }
    }

    /**
     Start observing secondary context presentation style which is in compact environment turns into fullscreen
     and otherwise looks like formsheet.

     In response to compact environment and fullscreen presentation, the observer re-assigns notification controller
     from primary to secondary context to mimic the look and feel of iPhone app. The opposite is also true, that it
     will make sure that notification controller is presented within primary context when secondary context is in
     formsheet presentation style.
     */
    private func addSecondaryContextPresentationStyleObserver() {
        removeSecondaryContextPresentationStyleObserver()

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(formSheetControllerWillChangeFullscreenPresentation(_:)),
            name: FormSheetPresentationController.willChangeFullScreenPresentation,
            object: secondaryNavigationContainer
        )
    }

    /**
     Stop observing secondary context presentation style.
     */
    private func removeSecondaryContextPresentationStyleObserver() {
        NotificationCenter.default.removeObserver(
            self,
            name: FormSheetPresentationController.willChangeFullScreenPresentation,
            object: secondaryNavigationContainer
        )
    }

    /**
     This method is called in response to changes in fullscreen presentation style of form sheet presentation
     controller.
     */
    @objc private func formSheetControllerWillChangeFullscreenPresentation(_ note: Notification) {
        guard let isFullscreenNumber = note
            .userInfo?[SecondaryContextPresentationController.isFullScreenUserInfoKey] as? NSNumber else { return }

        setNotificationControllerParent(isPrimary: !isFullscreenNumber.boolValue)
    }

    private var isPad: Bool {
        UIDevice.current.userInterfaceIdiom == .pad
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

        primaryNavigationContainer.notificationViewLayoutGuide = tunnelCoordinator.rootViewController.view
            .safeAreaLayoutGuide

        tunnelCoordinator.start()
        selectLocationCoordinator.start()
    }

    private func presentTOS(animated: Bool, completion: @escaping (Coordinator) -> Void) {
        let coordinator = TermsOfServiceCoordinator(navigationController: horizontalFlowController)

        coordinator.didFinish = { [weak self] _ in
            self?.appPreferences.isAgreedToTermsOfService = true
            self?.continueFlow(animated: true)
        }

        addChild(coordinator)
        coordinator.start()

        beginHorizontalFlow(animated: animated) {
            completion(coordinator)
        }
    }

    private func presentChangeLog(animated: Bool, completion: @escaping (Coordinator) -> Void) {
        let coordinator = ChangeLogCoordinator(interactor: ChangeLogInteractor())

        coordinator.didFinish = { [weak self] _ in
            self?.appPreferences.markChangeLogSeen()
            self?.router.dismiss(.changelog)
        }

        coordinator.start()

        presentChild(coordinator, animated: animated) {
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

        coordinator.didFinish = { [weak self] _ in
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

        coordinator.didFinishPayment = { [weak self] _ in
            guard let self = self else { return }

            if shouldDismissOutOfTime() {
                router.dismiss(.outOfTime, animated: true)
                continueFlow(animated: true)
            }
        }

        addChild(coordinator)
        coordinator.start(animated: animated)

        beginHorizontalFlow(animated: animated) {
            completion(coordinator)
        }
    }

    private func presentWelcome(animated: Bool, completion: @escaping (Coordinator) -> Void) {
        let coordinator = WelcomeCoordinator(
            navigationController: horizontalFlowController,
            storePaymentManager: storePaymentManager,
            tunnelManager: tunnelManager,
            accountsProxy: accountsProxy
        )
        coordinator.didFinish = { [weak self] in
            guard let self else { return }
            appPreferences.isShownOnboarding = true
            router.dismiss(.welcome, animated: false)
            continueFlow(animated: false)
        }
        coordinator.didLogout = { [weak self] preferredAccountNumber in
            guard let self else { return }
            router.dismissAll(.primary, animated: true)
            continueFlow(animated: true)
            preferredAccountNumberSubject.send(preferredAccountNumber)
        }

        addChild(coordinator)
        coordinator.start(animated: animated)

        beginHorizontalFlow(animated: animated) {
            completion(coordinator)
        }
    }

    private func shouldDismissOutOfTime() -> Bool {
        !(tunnelManager.deviceState.accountData?.isExpired ?? false)
    }

    private func presentSelectLocation(animated: Bool, completion: @escaping (Coordinator) -> Void) {
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

        coordinator.preferredAccountNumberPublisher = preferredAccountNumberSubject.eraseToAnyPublisher()

        coordinator.didFinish = { [weak self] _ in
            self?.continueFlow(animated: true)
        }
        coordinator.didCreateAccount = { [weak self] in
            self?.appPreferences.isShownOnboarding = false
        }

        addChild(coordinator)
        coordinator.start(animated: animated)

        beginHorizontalFlow(animated: animated) {
            completion(coordinator)
        }
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
            outgoingConnectionService: outgoingConnectionService
        )

        tunnelCoordinator.showSelectLocationPicker = { [weak self] in
            self?.router.present(.selectLocation, animated: true)
        }

        return tunnelCoordinator
    }

    private func makeSelectLocationCoordinator(forModalPresentation isModalPresentation: Bool)
        -> SelectLocationCoordinator {
        let navigationController = CustomNavigationController()
        navigationController.isNavigationBarHidden = !isModalPresentation

        let selectLocationCoordinator = SelectLocationCoordinator(
            navigationController: navigationController,
            tunnelManager: tunnelManager,
            relayCacheTracker: relayCacheTracker
        )

        selectLocationCoordinator.didFinish = { [weak self] _, _ in
            if isModalPresentation {
                self?.router.dismiss(.selectLocation, animated: true)
            }
        }

        return selectLocationCoordinator
    }

    private func presentAccount(animated: Bool, completion: @escaping (Coordinator) -> Void) {
        let accountInteractor = AccountInteractor(
            storePaymentManager: storePaymentManager,
            tunnelManager: tunnelManager,
            accountsProxy: accountsProxy
        )

        let coordinator = AccountCoordinator(
            navigationController: CustomNavigationController(),
            interactor: accountInteractor
        )

        coordinator.didFinish = { [weak self] _, reason in
            self?.didDismissAccount(reason)
        }

        coordinator.start(animated: animated)

        presentChild(
            coordinator,
            animated: animated,
            configuration: ModalPresentationConfiguration(
                preferredContentSize: UIMetrics.preferredFormSheetContentSize,
                modalPresentationStyle: .formSheet
            )
        ) { [weak self] in
            completion(coordinator)

            self?.onShowAccount?()
        }
    }

    private func presentSettings(
        route: SettingsNavigationRoute?,
        animated: Bool,
        completion: @escaping (Coordinator) -> Void
    ) {
        let interactorFactory = SettingsInteractorFactory(
            storePaymentManager: storePaymentManager,
            tunnelManager: tunnelManager,
            apiProxy: apiProxy,
            relayCacheTracker: relayCacheTracker
        )

        let navigationController = CustomNavigationController()

        let configurationTester = ProxyConfigurationTester(transportProvider: configuredTransportProvider)

        let coordinator = SettingsCoordinator(
            navigationController: navigationController,
            interactorFactory: interactorFactory,
            accessMethodRepository: accessMethodRepository,
            proxyConfigurationTester: configurationTester
        )

        coordinator.didFinish = { [weak self] _ in
            self?.router.dismissAll(.settings, animated: true)
        }

        coordinator.willNavigate = { [weak self] _, _, to in
            if to == .root {
                self?.onShowSettings?()
            }
        }

        coordinator.start(initialRoute: route)

        presentChild(
            coordinator,
            animated: animated,
            configuration: ModalPresentationConfiguration(
                preferredContentSize: UIMetrics.preferredFormSheetContentSize,
                modalPresentationStyle: .formSheet
            )
        ) {
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

        splitViewController.preferredDisplayMode = tunnelManager.deviceState.splitViewMode
    }

    private func deviceStateDidChange(_ deviceState: DeviceState, previousDeviceState: DeviceState) {
        splitViewController.preferredDisplayMode = deviceState.splitViewMode
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
        self.primaryNavigationContainer.update(configuration: rootDeviceInfoViewModel.configuration)
        self.secondaryNavigationContainer.update(configuration: rootDeviceInfoViewModel.configuration)
    }

    // MARK: - Out of time

    private func updateOutOfTimeTimer(accountData: StoredAccountData) {
        cancelOutOfTimeTimer()

        guard !accountData.isExpired else { return }

        let timer = Timer(fire: accountData.expiry, interval: 0, repeats: false, block: { [weak self] _ in
            self?.router.present(.outOfTime, animated: true)
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
        if isPad {
            return [.landscape, .portrait]
        } else {
            return [.portrait]
        }
    }

    func rootContainerViewAccessibilityPerformMagicTap(_ controller: RootContainerViewController)
        -> Bool {
        guard tunnelManager.deviceState.isLoggedIn else { return false }

        switch tunnelManager.tunnelStatus.state {
        case .connected, .connecting, .reconnecting, .waitingForConnectivity(.noConnection), .error:
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
        default: return
        }
    }

    // MARK: - Presenting

    var presentationContext: UIViewController {
        primaryNavigationContainer.presentedViewController ?? primaryNavigationContainer
    }
}

extension DeviceState {
    var splitViewMode: UISplitViewController.DisplayMode {
        isLoggedIn ? UISplitViewController.DisplayMode.oneBesideSecondary : .secondaryOnly
    }
}

fileprivate extension AppPreferencesDataSource {
    var hasSeenLastChanges: Bool {
        !ChangeLogInteractor().hasNewChanges ||
            (lastSeenChangeLogVersion == Bundle.main.shortVersion)
    }

    mutating func markChangeLogSeen() {
        lastSeenChangeLogVersion = Bundle.main.shortVersion
    }

    // swiftlint:disable:next file_length
}
