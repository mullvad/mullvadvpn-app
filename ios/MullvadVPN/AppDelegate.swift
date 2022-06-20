//
//  AppDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 19/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit
import StoreKit
import UserNotifications
import Logging
import BackgroundTasks

@UIApplicationMain
class AppDelegate: UIResponder, UIApplicationDelegate {

    var window: UIWindow?

    private var logger: Logger!

    #if targetEnvironment(simulator)
    private let simulatorTunnelProvider = SimulatorTunnelProviderHost()
    #endif

    private lazy var occlusionWindow: UIWindow = {
        let window = UIWindow(frame: UIScreen.main.bounds)
        window.rootViewController = LaunchViewController()
        window.windowLevel = .alert + 1
        return window
    }()

    private var rootContainer: RootContainerViewController?
    private var splitViewController: CustomSplitViewController?
    private var selectLocationViewController: SelectLocationViewController?
    private var connectController: ConnectViewController?
    private weak var settingsNavController: SettingsNavigationController?

    private var relayConstraints: RelayConstraints?

    private let operationQueue: AsyncOperationQueue = {
        let operationQueue = AsyncOperationQueue()
        operationQueue.maxConcurrentOperationCount = 1
        return operationQueue
    }()

    // MARK: - Application lifecycle

    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        // Setup logging
        initLoggingSystem(bundleIdentifier: Bundle.main.bundleIdentifier!)

        logger = Logger(label: "AppDelegate")

        #if targetEnvironment(simulator)
        // Configure mock tunnel provider on simulator
        SimulatorTunnelProvider.shared.delegate = simulatorTunnelProvider
        #endif

        if #available(iOS 13.0, *) {
            // Register background tasks on iOS 13
            registerBackgroundTasks()
        } else {
            // Set background refresh interval on iOS 12
            application.setMinimumBackgroundFetchInterval(
                ApplicationConfiguration.minimumBackgroundFetchInterval
            )
        }

        setupPaymentHandler()
        setupNotificationHandler()

        // Add relay cache observer
        RelayCache.Tracker.shared.addObserver(self)

        // Start initialization
        let setupTunnelManagerOperation = AsyncBlockOperation(dispatchQueue: .main) { blockOperation in
            TunnelManager.shared.loadConfiguration { error in
                dispatchPrecondition(condition: .onQueue(.main))

                if let error = error {
                    self.logger.error(chainedError: error, message: "Failed to load tunnels")

                    // TODO: avoid throwing fatal error and show the problem report UI instead.
                    fatalError(
                        error.displayChain(message: "Failed to load VPN tunnel configuration")
                    )
                }

                blockOperation.finish()
            }
        }

        let setupUIOperation = AsyncBlockOperation(dispatchQueue: .main) {
            self.logger.debug("Finished initialization. Show user interface.")

            self.relayConstraints = TunnelManager.shared.tunnelSettings?.relayConstraints

            self.rootContainer = RootContainerViewController()
            self.rootContainer?.delegate = self
            self.window?.rootViewController = self.rootContainer

            switch UIDevice.current.userInterfaceIdiom {
            case .pad:
                self.setupPadUI()

            case .phone:
                self.setupPhoneUI()

            default:
                fatalError()
            }

            NotificationManager.shared.updateNotifications()
            AppStorePaymentManager.shared.startPaymentQueueMonitoring()
        }

        operationQueue.addOperations([
            setupTunnelManagerOperation,
            setupUIOperation
        ], waitUntilFinished: false)

        // Create an app window
        self.window = UIWindow(frame: UIScreen.main.bounds)

        // Set an empty view controller while loading tunnels
        self.window?.rootViewController = LaunchViewController()

        // Show the window
        self.window?.makeKeyAndVisible()

        return true
    }

    func applicationDidBecomeActive(_ application: UIApplication) {
        // Refresh tunnel status.
        TunnelManager.shared.refreshTunnelStatus()

        // Start periodic relays updates
        RelayCache.Tracker.shared.startPeriodicUpdates()

        // Start periodic private key rotation
        TunnelManager.shared.startPeriodicPrivateKeyRotation()

        // Start periodic API address list updates
        AddressCache.Tracker.shared.startPeriodicUpdates()

        // Reveal application content
        occlusionWindow.isHidden = true
        window?.makeKeyAndVisible()
    }

    func applicationWillResignActive(_ application: UIApplication) {
        // Stop periodic relays updates
        RelayCache.Tracker.shared.stopPeriodicUpdates()

        // Stop periodic private key rotation
        TunnelManager.shared.stopPeriodicPrivateKeyRotation()

        // Stop periodic API address list updates
        AddressCache.Tracker.shared.stopPeriodicUpdates()

        // Hide application content
        occlusionWindow.makeKeyAndVisible()
    }

    func applicationDidEnterBackground(_ application: UIApplication) {
        if #available(iOS 13, *) {
            scheduleBackgroundTasks()
        }
    }

    func application(
        _ application: UIApplication,
        performFetchWithCompletionHandler completionHandler: @escaping (UIBackgroundFetchResult) -> Void
    )
    {
        logger.debug("Start background refresh.")

        let updateAddressCacheOperation = ResultBlockOperation<Bool, Error> { operation in
            let handle = AddressCache.Tracker.shared.updateEndpoints { completion in
                operation.finish(completion: completion)
            }

            operation.addCancellationBlock {
                handle.cancel()
            }
        }

        let updateRelaysOperation = ResultBlockOperation<RelayCache.FetchResult, RelayCache.Error>
        { operation in
            let handle = RelayCache.Tracker.shared.updateRelays { completion in
                operation.finish(completion: completion)
            }

            operation.addCancellationBlock {
                handle.cancel()
            }
        }

        let rotatePrivateKeyOperation = ResultBlockOperation<Bool, TunnelManager.Error>
        { operation in
            let handle = TunnelManager.shared.rotatePrivateKey(forceRotate: false) { completion in
                operation.finish(completion: completion)
            }

            operation.addCancellationBlock {
                handle.cancel()
            }
        }
        rotatePrivateKeyOperation.addDependencies([
            updateRelaysOperation,
            updateAddressCacheOperation
        ])

        let operations = [
            updateAddressCacheOperation,
            updateRelaysOperation,
            rotatePrivateKeyOperation
        ]

        let completeOperation = TransformOperation<UIBackgroundFetchResult, Void, Never>(
            dispatchQueue: .main
        )

        completeOperation.setExecutionBlock { backgroundFetchResult in
            self.logger.debug("Finish background refresh. Status: \(backgroundFetchResult).")

            completionHandler(backgroundFetchResult)
        }

        completeOperation.injectMany(context: [UIBackgroundFetchResult]())
            .injectCompletion(from: updateAddressCacheOperation, via: { results, completion in
                results.append(completion.backgroundFetchResult { $0 })
            })
            .injectCompletion(from: updateRelaysOperation, via: { results, completion in
                results.append(completion.backgroundFetchResult { $0 == .newContent })
            })
            .injectCompletion(from: rotatePrivateKeyOperation, via: { results, completion in
                results.append(completion.backgroundFetchResult { $0 })
            })
            .reduce { operationResults in
                let initialResult = operationResults.first ?? .failed
                let backgroundFetchResult = operationResults
                    .reduce(initialResult) { partialResult, other in
                        return partialResult.combine(with: other)
                    }

                return backgroundFetchResult
            }

        let groupOperation = GroupOperation(operations: operations)
        groupOperation.addObserver(
            BackgroundObserver(name: "Background refresh", cancelUponExpiration: true)
        )

        let operationQueue = AsyncOperationQueue()
        operationQueue.addOperation(groupOperation)
        operationQueue.addOperation(completeOperation)
    }

    // MARK: - Background tasks
    @available(iOS 13, *)
    private func registerBackgroundTasks() {
        registerAppRefreshTask()
        registerAddressCacheUpdateTask()
        registerKeyRotationTask()
    }

    @available(iOS 13.0, *)
    private func registerAppRefreshTask() {
        let isRegistered = BGTaskScheduler.shared.register(
            forTaskWithIdentifier: ApplicationConfiguration.appRefreshTaskIdentifier,
            using: nil
        ) { task in
            let handle = RelayCache.Tracker.shared.updateRelays { completion in
                task.setTaskCompleted(success: completion.isSuccess)
            }

            task.expirationHandler = {
                handle.cancel()
            }

            self.scheduleAppRefreshTask()
        }

        if isRegistered {
            logger.debug("Registered app refresh task.")
        } else {
            logger.error("Failed to register app refresh task.")
        }
    }

    @available(iOS 13.0, *)
    private func registerKeyRotationTask() {
        let isRegistered = BGTaskScheduler.shared.register(
            forTaskWithIdentifier: ApplicationConfiguration.privateKeyRotationTaskIdentifier,
            using: nil
        ) { task in
            let handle = TunnelManager.shared.rotatePrivateKey(forceRotate: false) { completion in
                self.scheduleKeyRotationTask()

                task.setTaskCompleted(success: completion.isSuccess)
            }

            task.expirationHandler = {
                handle.cancel()
            }
        }

        if isRegistered {
            logger.debug("Registered private key rotation task.")
        } else {
            logger.error("Failed to register private key rotation task.")
        }
    }

    @available(iOS 13.0, *)
    private func registerAddressCacheUpdateTask() {
        let isRegistered = BGTaskScheduler.shared.register(
            forTaskWithIdentifier: ApplicationConfiguration.addressCacheUpdateTaskIdentifier,
            using: nil
        ) { task in
            let handle = AddressCache.Tracker.shared.updateEndpoints { completion in
                self.scheduleAddressCacheUpdateTask()

                task.setTaskCompleted(success: completion.isSuccess)
            }

            task.expirationHandler = {
                handle.cancel()
            }
        }

        if isRegistered {
            logger.debug("Registered address cache update task.")
        } else {
            logger.error("Failed to register address cache update task.")
        }
    }

    @available(iOS 13.0, *)
    func scheduleBackgroundTasks() {
        scheduleAppRefreshTask()
        scheduleKeyRotationTask()
        scheduleAddressCacheUpdateTask()
    }

    @available(iOS 13.0, *)
    private func scheduleAppRefreshTask() {
        do {
            let date = RelayCache.Tracker.shared.getNextUpdateDate()

            let request = BGAppRefreshTaskRequest(
                identifier: ApplicationConfiguration.appRefreshTaskIdentifier
            )
            request.earliestBeginDate = date

            logger.debug("Schedule app refresh task at \(date.logFormatDate()).")

            try BGTaskScheduler.shared.submit(request)
        } catch {
            logger.error(
                chainedError: AnyChainedError(error),
                message: "Could not schedule app refresh task."
            )
        }
    }

    @available(iOS 13.0, *)
    private func scheduleKeyRotationTask() {
        do {
            guard let date = TunnelManager.shared.getNextKeyRotationDate() else {
                return
            }

            let request = BGProcessingTaskRequest(
                identifier: ApplicationConfiguration.privateKeyRotationTaskIdentifier
            )
            request.requiresNetworkConnectivity = true
            request.earliestBeginDate = date

            logger.debug("Schedule key rotation task at \(date.logFormatDate()).")

            try BGTaskScheduler.shared.submit(request)
        } catch {
            logger.error(
                chainedError: AnyChainedError(error),
                message: "Could not schedule private key rotation task."
            )
        }
    }

    @available(iOS 13.0, *)
    private func scheduleAddressCacheUpdateTask() {
        do {
            let date = AddressCache.Tracker.shared.nextScheduleDate()

            let request = BGProcessingTaskRequest(
                identifier: ApplicationConfiguration.addressCacheUpdateTaskIdentifier
            )
            request.requiresNetworkConnectivity = true
            request.earliestBeginDate = date

            logger.debug("Schedule address cache update task at \(date.logFormatDate()).")

            try BGTaskScheduler.shared.submit(request)
        } catch {
            logger.error(
                chainedError: AnyChainedError(error),
                message: "Could not schedule address cache update task."
            )
        }
    }

    // MARK: - Private

    private func setupPaymentHandler() {
        AppStorePaymentManager.shared.delegate = self
        AppStorePaymentManager.shared.addPaymentObserver(TunnelManager.shared)
    }

    private func setupNotificationHandler() {
        NotificationManager.shared.notificationProviders = [
            AccountExpiryNotificationProvider(),
            TunnelErrorNotificationProvider()
        ]
        UNUserNotificationCenter.current().delegate = self
    }

    private func setupPadUI() {
        let selectLocationController = makeSelectLocationController()
        let connectController = makeConnectViewController()

        let splitViewController = CustomSplitViewController()
        splitViewController.delegate = self
        splitViewController.minimumPrimaryColumnWidth = UIMetrics.minimumSplitViewSidebarWidth
        splitViewController.preferredPrimaryColumnWidthFraction = UIMetrics.maximumSplitViewSidebarWidthFraction
        splitViewController.primaryEdge = .trailing
        splitViewController.dividerColor = UIColor.MainSplitView.dividerColor
        splitViewController.viewControllers = [selectLocationController, connectController]

        self.selectLocationViewController = selectLocationController
        self.splitViewController = splitViewController
        self.connectController = connectController

        self.rootContainer?.setViewControllers([splitViewController], animated: false)
        showSplitViewMaster(TunnelManager.shared.isAccountSet, animated: false)

        let rootContainerWrapper = makeLoginContainerController()

        if !isAgreedToTermsOfService() {
            let consentViewController = self.makeConsentController { [weak self] (viewController) in
                guard let self = self else { return }

                if TunnelManager.shared.isAccountSet {
                    rootContainerWrapper.dismiss(animated: true) {
                        self.showAccountSettingsControllerIfAccountExpired()
                    }
                } else {
                    rootContainerWrapper.pushViewController(self.makeLoginController(), animated: true)
                }
            }
            rootContainerWrapper.setViewControllers([consentViewController], animated: false)
            self.rootContainer?.present(rootContainerWrapper, animated: false)
        } else if !TunnelManager.shared.isAccountSet {
            rootContainerWrapper.setViewControllers([makeLoginController()], animated: false)
            self.rootContainer?.present(rootContainerWrapper, animated: false)
        } else {
            self.showAccountSettingsControllerIfAccountExpired()
        }
    }

    private func setupPhoneUI() {
        let showNextController = { [weak self] (_ animated: Bool) in
            guard let self = self else { return }

            let loginViewController = self.makeLoginController()
            var viewControllers: [UIViewController] = [loginViewController]

            if TunnelManager.shared.isAccountSet {
                let connectController = self.makeConnectViewController()
                viewControllers.append(connectController)
                self.connectController = connectController
            }

            self.rootContainer?.setViewControllers(viewControllers, animated: animated) {
                self.showAccountSettingsControllerIfAccountExpired()
            }
        }

        if isAgreedToTermsOfService() {
            showNextController(false)
        } else {
            let consentViewController = self.makeConsentController { (consentController) in
                showNextController(true)
            }

            self.rootContainer?.setViewControllers([consentViewController], animated: false)
        }
    }

    private func makeConnectViewController() -> ConnectViewController {
        let connectController = ConnectViewController()
        connectController.delegate = self
        NotificationManager.shared.delegate = self

        return connectController
    }

    private func makeSelectLocationController() -> SelectLocationViewController {
        let selectLocationController = SelectLocationViewController()
        selectLocationController.delegate = self

        if let cachedRelays = RelayCache.Tracker.shared.getCachedRelays() {
            selectLocationController.setCachedRelays(cachedRelays)
        }

        if let relayLocation = relayConstraints?.location.value {
            selectLocationController.setSelectedRelayLocation(
                relayLocation,
                animated: false,
                scrollPosition: .middle
            )
        }

        return selectLocationController
    }

    private func makeConsentController(completion: @escaping (UIViewController) -> Void) -> ConsentViewController {
        let consentViewController = ConsentViewController()

        if UIDevice.current.userInterfaceIdiom == .pad {
            consentViewController.modalPresentationStyle = .formSheet
            if #available(iOS 13.0, *) {
                consentViewController.isModalInPresentation = true
            }
        }

        consentViewController.completionHandler = { (consentViewController) in
            setAgreedToTermsOfService()
            completion(consentViewController)
        }

        return consentViewController
    }

    private func makeLoginContainerController() -> RootContainerViewController {
        let rootContainerWrapper = RootContainerViewController()
        rootContainerWrapper.delegate = self
        rootContainerWrapper.preferredContentSize = CGSize(width: 480, height: 600)

        if UIDevice.current.userInterfaceIdiom == .pad {
            rootContainerWrapper.modalPresentationStyle = .formSheet
            if #available(iOS 13.0, *) {
                // Prevent swiping off the login or consent controllers
                rootContainerWrapper.isModalInPresentation = true
            }
        }

        rootContainerWrapper.presentationController?.delegate = self

        return rootContainerWrapper
    }

    private func makeLoginController() -> LoginViewController {
        let controller = LoginViewController()
        controller.delegate = self
        return controller
    }

    private func makeSettingsNavigationController(route: SettingsNavigationRoute?) -> SettingsNavigationController {
        let navController = SettingsNavigationController()
        navController.settingsDelegate = self

        if UIDevice.current.userInterfaceIdiom == .pad {
            navController.preferredContentSize = CGSize(width: 480, height: 568)
            navController.modalPresentationStyle = .formSheet
        }

        navController.presentationController?.delegate = navController

        if let route = route {
            navController.navigate(to: route, animated: false)
        }

        return navController
    }

    private func showAccountSettingsControllerIfAccountExpired() {
        guard let accountExpiry = TunnelManager.shared.accountExpiry, accountExpiry <= Date() else { return }

        rootContainer?.showSettings(navigateTo: .account, animated: true)
    }

    private func showSplitViewMaster(_ show: Bool, animated: Bool) {
        if show {
            splitViewController?.preferredDisplayMode = .allVisible
            connectController?.setMainContentHidden(false, animated: animated)
        } else {
            splitViewController?.preferredDisplayMode = .primaryHidden
            connectController?.setMainContentHidden(true, animated: animated)
        }
    }
}

// MARK: - RootContainerViewControllerDelegate

extension AppDelegate: RootContainerViewControllerDelegate {

    func rootContainerViewControllerShouldShowSettings(_ controller: RootContainerViewController, navigateTo route: SettingsNavigationRoute?, animated: Bool) {
        // Check if settings controller is already presented.
        if let settingsNavController = self.settingsNavController {
            if let route = route {
                settingsNavController.navigate(to: route, animated: animated)
            } else {
                settingsNavController.popToRootViewController(animated: animated)
            }
        } else {
            let navController = makeSettingsNavigationController(route: route)

            // On iPad the login controller can be presented modally above the root container.
            // in that case we have to use the presented controller to present the next modal.
            if let presentedController = controller.presentedViewController {
                presentedController.present(navController, animated: true)
            } else {
                controller.present(navController, animated: true)
            }

            // Save the reference for later.
            self.settingsNavController = navController
        }
    }

    func rootContainerViewSupportedInterfaceOrientations(_ controller: RootContainerViewController) -> UIInterfaceOrientationMask {
        switch UIDevice.current.userInterfaceIdiom {
        case .pad:
            return [.landscape, .portrait]
        case .phone:
            return [.portrait]
        default:
            return controller.supportedInterfaceOrientations
        }
    }

    func rootContainerViewAccessibilityPerformMagicTap(_ controller: RootContainerViewController) -> Bool {
        guard TunnelManager.shared.isAccountSet else { return false }

        switch TunnelManager.shared.tunnelState {
        case .connected, .connecting, .reconnecting:
            TunnelManager.shared.reconnectTunnel(completionHandler: nil)
        case .disconnecting, .disconnected:
            TunnelManager.shared.startTunnel()
        case .pendingReconnect:
            break
        }
        return true
    }
}

// MARK: - NotificationManagerDelegate
extension AppDelegate: NotificationManagerDelegate {
    func notificationManagerDidUpdateInAppNotifications(_ manager: NotificationManager, notifications: [InAppNotificationDescriptor]) {
        connectController?.notificationController.setNotifications(notifications, animated: true)
    }
}

// MARK: - LoginViewControllerDelegate

extension AppDelegate: LoginViewControllerDelegate {

    func loginViewController(_ controller: LoginViewController, loginWithAccountToken accountNumber: String, completion: @escaping (OperationCompletion<StoredAccountData?, TunnelManager.Error>) -> Void) {
        self.rootContainer?.setEnableSettingsButton(false)

        TunnelManager.shared.setAccount(action: .existing(accountNumber)) { operationCompletion in
            switch operationCompletion {
            case .success:
                self.logger.debug("Logged in with existing account.")
                // RootContainer's settings button will be re-enabled in `loginViewControllerDidLogin`

            case .failure(let error):
                self.logger.error(chainedError: error, message: "Failed to log in with existing account.")
                fallthrough

            case .cancelled:
                self.rootContainer?.setEnableSettingsButton(true)
            }

            completion(operationCompletion)
        }
    }

    func loginViewControllerLoginWithNewAccount(_ controller: LoginViewController, completion: @escaping (OperationCompletion<StoredAccountData?, TunnelManager.Error>) -> Void) {
        self.rootContainer?.setEnableSettingsButton(false)

        TunnelManager.shared.setAccount(action: .new) { operationCompletion in
            switch operationCompletion {
            case .success:
                self.logger.debug("Logged in with new account number.")
                // RootContainer's settings button will be re-enabled in `loginViewControllerDidLogin`

            case .failure(let error):
                self.logger.error(chainedError: error, message: "Failed to log in with new account.")
                fallthrough

            case .cancelled:
                self.rootContainer?.setEnableSettingsButton(true)
            }

            completion(operationCompletion)
        }
    }

    func loginViewControllerDidLogin(_ controller: LoginViewController) {
        self.window?.isUserInteractionEnabled = false

        // Move the settings button back into header bar
        self.rootContainer?.removeSettingsButtonFromPresentationContainer()

        self.relayConstraints = TunnelManager.shared.tunnelSettings?.relayConstraints
        self.selectLocationViewController?.setSelectedRelayLocation(relayConstraints?.location.value, animated: false, scrollPosition: .middle)

        switch UIDevice.current.userInterfaceIdiom {
        case .phone:
            let connectController = self.makeConnectViewController()
            self.rootContainer?.pushViewController(connectController, animated: true) {
                self.showAccountSettingsControllerIfAccountExpired()
            }
            self.connectController = connectController
        case .pad:
            self.showSplitViewMaster(true, animated: true)

            controller.dismiss(animated: true) {
                self.showAccountSettingsControllerIfAccountExpired()
            }
        default:
            fatalError()
        }

        self.window?.isUserInteractionEnabled = true
        self.rootContainer?.setEnableSettingsButton(true)
    }

}

// MARK: - SettingsNavigationControllerDelegate

extension AppDelegate: SettingsNavigationControllerDelegate {

    func settingsNavigationController(_ controller: SettingsNavigationController, didFinishWithReason reason: SettingsDismissReason) {
        switch UIDevice.current.userInterfaceIdiom {
        case .phone:
            if case .userLoggedOut = reason {
                rootContainer?.popToRootViewController(animated: false)

                let loginController = rootContainer?.topViewController as? LoginViewController

                loginController?.reset()
            }
            controller.dismiss(animated: true)

        case .pad:
            if case .userLoggedOut = reason {
                self.showSplitViewMaster(false, animated: true)
            }

            controller.dismiss(animated: true) {
                if case .userLoggedOut = reason {
                    let rootContainerWrapper = self.makeLoginContainerController()
                    rootContainerWrapper.setViewControllers([self.makeLoginController()], animated: false)
                    self.rootContainer?.present(rootContainerWrapper, animated: true)
                }
            }

        default:
            fatalError()
        }

    }

}

// MARK: - ConnectViewControllerDelegate

extension AppDelegate: ConnectViewControllerDelegate {

    func connectViewControllerShouldShowSelectLocationPicker(_ controller: ConnectViewController) {
        let contentController = makeSelectLocationController()
        contentController.navigationItem.largeTitleDisplayMode = .never
        contentController.navigationItem.rightBarButtonItem = UIBarButtonItem(barButtonSystemItem: .done, target: self, action: #selector(handleDismissSelectLocationController(_:)))

        let navController = SelectLocationNavigationController(contentController: contentController)
        self.rootContainer?.present(navController, animated: true)
        self.selectLocationViewController = contentController
    }

    @objc private func handleDismissSelectLocationController(_ sender: Any) {
        self.selectLocationViewController?.dismiss(animated: true)
    }
}

// MARK: - SelectLocationViewControllerDelegate

extension AppDelegate: SelectLocationViewControllerDelegate {
    func selectLocationViewController(_ controller: SelectLocationViewController, didSelectRelayLocation relayLocation: RelayLocation) {
        // Dismiss view controller in modal presentation
        if controller.presentingViewController != nil {
            self.window?.isUserInteractionEnabled = false
            DispatchQueue.main.asyncAfter(deadline: .now() + .milliseconds(250)) {
                self.window?.isUserInteractionEnabled = true
                controller.dismiss(animated: true) {
                    self.selectLocationControllerDidSelectRelayLocation(relayLocation)
                }
            }
        } else {
            selectLocationControllerDidSelectRelayLocation(relayLocation)
        }
    }

    private func selectLocationControllerDidSelectRelayLocation(_ relayLocation: RelayLocation) {
        let relayConstraints = RelayConstraints(location: .only(relayLocation))

        TunnelManager.shared.setRelayConstraints(relayConstraints) { error in
            self.relayConstraints = relayConstraints

            if let error = error {
                self.logger.error(chainedError: error, message: "Failed to update relay constraints")
            } else {
                self.logger.debug("Updated relay constraints: \(relayConstraints)")
                TunnelManager.shared.startTunnel()
            }
        }
    }
}

// MARK: - UIAdaptivePresentationControllerDelegate

extension AppDelegate: UIAdaptivePresentationControllerDelegate {

    func adaptivePresentationStyle(for controller: UIPresentationController, traitCollection: UITraitCollection) -> UIModalPresentationStyle {
        if controller.presentedViewController is RootContainerViewController {
            // Use .formSheet presentation in regular horizontal environment and .fullScreen
            // in compact environment.
            if traitCollection.horizontalSizeClass == .regular {
                return .formSheet
            } else {
                return .fullScreen
            }
        } else {
            return .none
        }
    }

    func presentationController(_ presentationController: UIPresentationController, willPresentWithAdaptiveStyle style: UIModalPresentationStyle, transitionCoordinator: UIViewControllerTransitionCoordinator?) {
        let actualStyle: UIModalPresentationStyle

        // When adaptive presentation is not changing, the `style` is set to `.none`
        if case .none = style {
            actualStyle = presentationController.presentedViewController.modalPresentationStyle
        } else {
            actualStyle = style
        }

        // Force hide header bar in .formSheet presentation and show it in .fullScreen presentation
        if let wrapper = presentationController.presentedViewController as? RootContainerViewController {
            wrapper.setOverrideHeaderBarHidden(actualStyle == .formSheet, animated: false)
        }

        guard actualStyle == .formSheet else {
            // Move the settings button back into header bar
            self.rootContainer?.removeSettingsButtonFromPresentationContainer()

            return
        }

        // Add settings button into the modal container to make it accessible by user
        if let transitionCoordinator = transitionCoordinator {
            transitionCoordinator.animate(alongsideTransition: { (context) in
                self.rootContainer?.addSettingsButtonToPresentationContainer(context.containerView)
            }, completion: { (context) in
                // no-op
            })
        } else {
            if let containerView = presentationController.containerView {
                rootContainer?.addSettingsButtonToPresentationContainer(containerView)
            } else {
                logger.warning("Cannot obtain the containerView for presentation controller when presenting with adaptive style \(actualStyle.rawValue) and missing transition coordinator.")
            }
        }
    }
}

// MARK: - RelayCacheObserver

extension AppDelegate: RelayCacheObserver {

    func relayCache(_ relayCache: RelayCache.Tracker, didUpdateCachedRelays cachedRelays: RelayCache.CachedRelays) {
        selectLocationViewController?.setCachedRelays(cachedRelays)
    }

}

// MARK: - AppStorePaymentManagerDelegate

extension AppDelegate: AppStorePaymentManagerDelegate {

    func appStorePaymentManager(_ manager: AppStorePaymentManager,
                                didRequestAccountTokenFor payment: SKPayment) -> String?
    {
        // Since we do not persist the relation between the payment and account token between the
        // app launches, we assume that all successful purchases belong to the active account token.
        return TunnelManager.shared.tunnelSettings?.account.number
    }

}


// MARK: - UISplitViewControllerDelegate

extension AppDelegate: UISplitViewControllerDelegate {

    func primaryViewController(forExpanding splitViewController: UISplitViewController) -> UIViewController? {
        // Restore the select location controller as primary when expanding the split view
        return selectLocationViewController
    }

    func primaryViewController(forCollapsing splitViewController: UISplitViewController) -> UIViewController? {
        // Set the connect controller as primary when collapsing the split view
        return connectController
    }

    func splitViewController(_ splitViewController: UISplitViewController, separateSecondaryFrom primaryViewController: UIViewController) -> UIViewController? {
        // Dismiss the select location controller when expanding the split view
        if self.selectLocationViewController?.presentingViewController != nil {
            self.selectLocationViewController?.dismiss(animated: false)
        }
        return nil
    }

}

// MARK: - UNUserNotificationCenterDelegate

extension AppDelegate: UNUserNotificationCenterDelegate {

    func userNotificationCenter(_ center: UNUserNotificationCenter, didReceive response: UNNotificationResponse, withCompletionHandler completionHandler: @escaping () -> Void) {
        let blockOperation = AsyncBlockOperation(dispatchQueue: .main) {
            if response.notification.request.identifier == accountExpiryNotificationIdentifier,
               response.actionIdentifier == UNNotificationDefaultActionIdentifier {
                self.rootContainer?.showSettings(navigateTo: .account, animated: true)
            }

            completionHandler()
        }

        operationQueue.addOperation(blockOperation)
    }

    func userNotificationCenter(_ center: UNUserNotificationCenter, willPresent notification: UNNotification, withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void) {
        if #available(iOS 14.0, *) {
            completionHandler([.list])
        } else {
            completionHandler([])
        }
    }

}

// MARK: -

/// A enum holding the `UserDefaults` string keys
private let isAgreedToTermsOfServiceKey = "isAgreedToTermsOfService"

/// Returns true if user agreed to terms of service, otherwise false.
func isAgreedToTermsOfService() -> Bool {
    return UserDefaults.standard.bool(forKey: isAgreedToTermsOfServiceKey)
}

/// Save the boolean flag in preferences indicating that the user agreed to terms of service.
func setAgreedToTermsOfService() {
    UserDefaults.standard.set(true, forKey: isAgreedToTermsOfServiceKey)
}
