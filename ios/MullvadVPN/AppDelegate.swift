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

@UIApplicationMain
class AppDelegate: UIResponder, UIApplicationDelegate {

    var window: UIWindow?

    private var logger: Logger?

    #if targetEnvironment(simulator)
    private let simulatorTunnelProvider = SimulatorTunnelProviderHost()
    #endif

    #if DEBUG
    private let packetTunnelLogForwarder = LogStreamer<UTF8>(fileURLs: [ApplicationConfiguration.packetTunnelLogFileURL!])
    #endif

    private var rootContainer: RootContainerViewController?
    private var splitViewController: CustomSplitViewController?
    private var selectLocationViewController: SelectLocationViewController?
    private var connectController: ConnectViewController?
    private weak var settingsNavController: SettingsNavigationController?

    private var cachedRelays: CachedRelays? {
        didSet {
            if let cachedRelays = cachedRelays {
                self.selectLocationViewController?.setCachedRelays(cachedRelays)
            }
        }
    }
    private var relayConstraints: RelayConstraints?
    private let alertPresenter = AlertPresenter()

    private let notificationManager = NotificationManager()

    // MARK: - Application lifecycle

    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        // Setup logging
        initLoggingSystem(bundleIdentifier: Bundle.main.bundleIdentifier!)

        self.logger = Logger(label: "AppDelegate")

        #if DEBUG
        let stdoutStream = TextFileOutputStream.standardOutputStream()
        packetTunnelLogForwarder.start { (str) in
            stdoutStream.write("\(str)\n")
        }
        #endif

        #if targetEnvironment(simulator)
        // Configure mock tunnel provider on simulator
        SimulatorTunnelProvider.shared.delegate = simulatorTunnelProvider
        #endif

        // Assign user notification center delegate
        UNUserNotificationCenter.current().delegate = self

        // Create an app window
        self.window = UIWindow(frame: UIScreen.main.bounds)

        // Set an empty view controller while loading tunnels
        let launchController = UIViewController()
        launchController.view.backgroundColor = .primaryColor
        self.window?.rootViewController = launchController

        // Update relays
        RelayCache.shared.addObserver(self)
        RelayCache.shared.updateRelays()

        // Load initial relays
        self.logger?.debug("Load relays")
        RelayCache.shared.read { (result) in
            DispatchQueue.main.async {
                switch result {
                case .success(let cachedRelays):
                    self.cachedRelays = cachedRelays
                    self.logger?.debug("Loaded relays")

                case .failure(let error):
                    self.logger?.error(chainedError: error, message: "Failed to load initial relays")
                }
            }
        }

        // Load tunnels
        self.logger?.debug("Load tunnels")
        TunnelManager.shared.loadTunnel(accountToken: Account.shared.token) { (result) in
            DispatchQueue.main.async {
                switch result {
                case .success:
                    self.logger?.debug("Loaded tunnels")
                    self.relayConstraints = TunnelManager.shared.tunnelSettings?.relayConstraints
                    self.didFinishInitialization()

                case .failure(let error):
                    self.logger?.error(chainedError: error, message: "Failed to load tunnels")

                    switch error {
                    case .loadAllVPNConfigurations(_), .removeInconsistentVPNConfiguration(_):
                        // TODO: avoid throwing fatal error and show the problem report UI instead.
                        fatalError(error.displayChain(message: "Failed to load tunnels"))

                    case .migrateTunnelSettings(_), .readTunnelSettings(_):
                        // Forget that user was logged in since tunnel settings are likely corrupt
                        // or missing.
                        Account.shared.forget {
                            self.didFinishInitialization()
                        }

                    default:
                        fatalError("Unexpected error coming from loadTunnel()")
                    }
                }
            }
        }

        // Show the window
        self.window?.makeKeyAndVisible()

        return true
    }

    func applicationDidBecomeActive(_ application: UIApplication) {
        TunnelManager.shared.refreshTunnelState(completionHandler: nil)
    }

    // MARK: - Private

    private func didFinishInitialization() {
        self.logger?.debug("Finished initialization. Show user interface.")

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

        notificationManager.delegate = connectController?.notificationController
        notificationManager.notificationProviders = [
            AccountExpiryNotificationProvider()
        ]
        notificationManager.updateNotifications()

        startPaymentQueueHandling()
    }

    private func startPaymentQueueHandling() {
        let paymentManager = AppStorePaymentManager.shared
        paymentManager.delegate = self

        Account.shared.startPaymentMonitoring(with: paymentManager)
        paymentManager.startPaymentQueueMonitoring()
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
        showSplitViewMaster(Account.shared.isLoggedIn, animated: false)

        let rootContainerWrapper = makeLoginContainerController()

        if !Account.shared.isAgreedToTermsOfService {
            let consentViewController = self.makeConsentController { [weak self] (viewController) in
                guard let self = self else { return }

                if Account.shared.isLoggedIn {
                    rootContainerWrapper.dismiss(animated: true) {
                        self.showAccountSettingsControllerIfAccountExpired()
                    }
                } else {
                    rootContainerWrapper.pushViewController(self.makeLoginController(), animated: true)
                }
            }
            rootContainerWrapper.setViewControllers([consentViewController], animated: false)
            self.rootContainer?.present(rootContainerWrapper, animated: false)
        } else if !Account.shared.isLoggedIn {
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

            if Account.shared.isLoggedIn {
                let connectController = self.makeConnectViewController()
                viewControllers.append(connectController)
                self.connectController = connectController
            }

            self.rootContainer?.setViewControllers(viewControllers, animated: animated) {
                self.showAccountSettingsControllerIfAccountExpired()
            }
        }

        if Account.shared.isAgreedToTermsOfService {
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

        return connectController
    }

    private func makeSelectLocationController() -> SelectLocationViewController {
        let selectLocationController = SelectLocationViewController()
        selectLocationController.delegate = self

        if let cachedRelays = cachedRelays {
            selectLocationController.setCachedRelays(cachedRelays)
        }

        if let relayLocation = relayConstraints?.location.value {
            selectLocationController.setSelectedRelayLocation(relayLocation, animated: false, scrollPosition: .middle)
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
            Account.shared.agreeToTermsOfService()
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
        guard let accountExpiry = Account.shared.expiry, AccountExpiry(date: accountExpiry).isExpired else { return }

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
        guard Account.shared.isLoggedIn else { return false }

        switch TunnelManager.shared.tunnelState {
        case .connected, .connecting, .reconnecting:
            reconnectTunnel()
        case .disconnecting, .disconnected:
            connectTunnel()
        }
        return true
    }
}

// MARK: - LoginViewControllerDelegate

extension AppDelegate: LoginViewControllerDelegate {

    func loginViewController(_ controller: LoginViewController, loginWithAccountToken accountToken: String, completion: @escaping (Result<AccountResponse, Account.Error>) -> Void) {
        self.rootContainer?.setEnableSettingsButton(false)

        Account.shared.login(with: accountToken) { (result) in
            switch result {
            case .success:
                self.logger?.debug("Logged in with existing token")
                // RootContainer's settings button will be re-enabled in `loginViewControllerDidLogin`

            case .failure(let error):
                self.logger?.error(chainedError: error, message: "Failed to log in with existing account")
                self.rootContainer?.setEnableSettingsButton(true)
            }

            completion(result)
        }
    }

    func loginViewControllerLoginWithNewAccount(_ controller: LoginViewController, completion: @escaping (Result<AccountResponse, Account.Error>) -> Void) {
        self.rootContainer?.setEnableSettingsButton(false)

        Account.shared.loginWithNewAccount { (result) in
            switch result {
            case .success:
                self.logger?.debug("Logged in with new account token")
                // RootContainer's settings button will be re-enabled in `loginViewControllerDidLogin`

            case .failure(let error):
                self.logger?.error(chainedError: error, message: "Failed to log in with new account")
                self.rootContainer?.setEnableSettingsButton(true)
            }

            completion(result)
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

    func connectViewControllerShouldConnectTunnel(_ controller: ConnectViewController) {
        connectTunnel()
    }

    func connectViewControllerShouldDisconnectTunnel(_ controller: ConnectViewController) {
        disconnectTunnel()
    }

    func connectViewControllerShouldReconnectTunnel(_ controller: ConnectViewController) {
        reconnectTunnel()
    }

    @objc private func handleDismissSelectLocationController(_ sender: Any) {
        self.selectLocationViewController?.dismiss(animated: true)
    }

    private func connectTunnel() {
        TunnelManager.shared.startTunnel { (result) in
            DispatchQueue.main.async {
                switch result {
                case .success:
                    self.logger?.debug("Connected VPN tunnel")

                case .failure(let error):
                    self.logger?.error(chainedError: error, message: "Failed to start the VPN tunnel")
                    self.presentTunnelError(error, alertTitle: NSLocalizedString("Failed to start the VPN tunnel", comment: ""))
                }
            }
        }
    }

    private func disconnectTunnel() {
        TunnelManager.shared.stopTunnel { (result) in
            switch result {
            case .success:
                self.logger?.debug("Disconnected VPN tunnel")

            case .failure(let error):
                self.logger?.error(chainedError: error, message: "Failed to stop the VPN tunnel")
                self.presentTunnelError(error, alertTitle: NSLocalizedString("Failed to stop the VPN tunnel", comment: ""))
            }
        }
    }

    private func reconnectTunnel() {
        TunnelManager.shared.reconnectTunnel {
            self.logger?.debug("Re-connected VPN tunnel")
        }
    }

    private func presentTunnelError(_ error: TunnelManager.Error, alertTitle: String) {
        let alertController = UIAlertController(
            title: alertTitle,
            message: error.errorChainDescription,
            preferredStyle: .alert
        )
        alertController.addAction(
            UIAlertAction(title: NSLocalizedString("OK", comment: ""), style: .cancel)
        )

        self.alertPresenter.enqueue(alertController, presentingController: self.rootContainer!)
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

        TunnelManager.shared.setRelayConstraints(relayConstraints) { [weak self] (result) in
            guard let self = self else { return }

            DispatchQueue.main.async {
                self.relayConstraints = relayConstraints

                switch result {
                case .success:
                    self.logger?.debug("Updated relay constraints: \(relayConstraints)")
                    self.connectTunnel()

                case .failure(let error):
                    self.logger?.error(chainedError: error, message: "Failed to update relay constraints")
                }
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
                self.rootContainer?.addSettingsButtonToPresentationContainer(containerView)
            } else {
                logger?.warning("Cannot obtain the containerView for presentation controller when presenting with adaptive style \(actualStyle.rawValue) and missing transition coordinator.")
            }
        }
    }
}

// MARK: - RelayCacheObserver

extension AppDelegate: RelayCacheObserver {

    func relayCache(_ relayCache: RelayCache, didUpdateCachedRelays cachedRelays: CachedRelays) {
        DispatchQueue.main.async {
            self.cachedRelays = cachedRelays
        }
    }

}

// MARK: - AppStorePaymentManagerDelegate

extension AppDelegate: AppStorePaymentManagerDelegate {

    func appStorePaymentManager(_ manager: AppStorePaymentManager,
                                didRequestAccountTokenFor payment: SKPayment) -> String?
    {
        // Since we do not persist the relation between the payment and account token between the
        // app launches, we assume that all successful purchases belong to the active account token.
        return Account.shared.token
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
        if response.notification.request.identifier == kAccountExpiryNotificationIdentifier,
           response.actionIdentifier == UNNotificationDefaultActionIdentifier {
            rootContainer?.showSettings(navigateTo: .account, animated: true)
        }

        completionHandler()
    }

    func userNotificationCenter(_ center: UNUserNotificationCenter, willPresent notification: UNNotification, withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void) {
        if #available(iOS 14.0, *) {
            completionHandler([.list])
        } else {
            completionHandler([])
        }
    }

}
