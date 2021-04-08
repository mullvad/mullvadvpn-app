//
//  AppDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 19/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit
import StoreKit
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
    private var selectLocationViewController: SelectLocationViewController?
    private var connectController: ConnectViewController?

    private var cachedRelays: CachedRelays? {
        didSet {
            if let cachedRelays = cachedRelays {
                self.selectLocationViewController?.setCachedRelays(cachedRelays)
            }
        }
    }
    private var relayConstraints: RelayConstraints?
    private let alertPresenter = AlertPresenter()

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
        RelayCache.shared.read { (result) in
            DispatchQueue.main.async {
                switch result {
                case .success(let cachedRelays):
                    self.cachedRelays = cachedRelays

                case .failure(let error):
                    self.logger?.error(chainedError: error, message: "Failed to fetch initial relays")
                }
            }
        }

        // Load tunnels
        TunnelManager.shared.loadTunnel(accountToken: Account.shared.token) { (result) in
            DispatchQueue.main.async {
                if case .failure(let error) = result {
                    fatalError(error.displayChain(message: "Failed to load the tunnel for account"))
                }

                TunnelManager.shared.getRelayConstraints { (result) in
                    DispatchQueue.main.async {
                        switch result {
                        case .success(let relayConstraints):
                            self.relayConstraints = relayConstraints

                        case .failure(let error):
                            self.logger?.error(chainedError: error, message: "Failed to load relay constraints")
                        }

                        self.rootContainer = RootContainerViewController()
                        self.rootContainer?.delegate = self
                        self.window?.rootViewController = self.rootContainer

                        self.setupPhoneUI()
                    }
                }
            }
        }

        // Show the window
        self.window?.makeKeyAndVisible()

        startPaymentQueueHandling()

        return true
    }

    func applicationDidBecomeActive(_ application: UIApplication) {
        TunnelManager.shared.refreshTunnelState(completionHandler: nil)
    }

    // MARK: - Private

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

        consentViewController.completionHandler = { (consentViewController) in
            Account.shared.agreeToTermsOfService()
            completion(consentViewController)
        }

        return consentViewController
    }

    private func makeLoginController() -> LoginViewController {
        let controller = LoginViewController()
        controller.delegate = self

        return controller
    }

    private func makeSettingsNavigationController(route: SettingsNavigationRoute?) -> SettingsNavigationController {
        let navController = SettingsNavigationController()
        navController.settingsDelegate = self
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

    private func startPaymentQueueHandling() {
        let paymentManager = AppStorePaymentManager.shared
        paymentManager.delegate = self
        paymentManager.startPaymentQueueMonitoring()

        Account.shared.startPaymentMonitoring(with: paymentManager)
    }

}

// MARK: - RootContainerViewControllerDelegate

extension AppDelegate: RootContainerViewControllerDelegate {
    func rootContainerViewControllerShouldShowSettings(_ controller: RootContainerViewController, navigateTo route: SettingsNavigationRoute?, animated: Bool) {
        let navController = makeSettingsNavigationController(route: route)

        // On iPad the login controller can be presented modally above the root container.
        // in that case we have to use the presented controller to present the next modal.
        if let presentedController = controller.presentedViewController {
            presentedController.present(navController, animated: true)
        } else {
            controller.present(navController, animated: true)
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

        TunnelManager.shared.getRelayConstraints { [weak self] (result) in
            guard let self = self else { return }

            DispatchQueue.main.async {
                switch result {
                case .success(let relayConstraints):
                    self.relayConstraints = relayConstraints
                    self.selectLocationViewController?.setSelectedRelayLocation(relayConstraints.location.value, animated: false, scrollPosition: .middle)

                case .failure(let error):
                    self.logger?.error(chainedError: error, message: "Failed to load relay constraints after log in")
                }

                let connectController = self.makeConnectViewController()
                self.rootContainer?.pushViewController(connectController, animated: true) {
                    self.showAccountSettingsControllerIfAccountExpired()
                }
                self.connectController = connectController

                self.window?.isUserInteractionEnabled = true
                self.rootContainer?.setEnableSettingsButton(true)
            }
        }
    }

}

// MARK: - SettingsNavigationControllerDelegate

extension AppDelegate: SettingsNavigationControllerDelegate {

    func settingsNavigationController(_ controller: SettingsNavigationController, didFinishWithReason reason: SettingsDismissReason) {
        if case .userLoggedOut = reason {
            rootContainer?.popToRootViewController(animated: false)

            let loginController = rootContainer?.topViewController as? LoginViewController

            loginController?.reset()
        }
        controller.dismiss(animated: true)
    }

}

// MARK: - ConnectViewControllerDelegate

extension AppDelegate: ConnectViewControllerDelegate {

    func connectViewControllerShouldShowSelectLocationPicker(_ controller: ConnectViewController) {
        let contentController = makeSelectLocationController()
        contentController.navigationItem.title = NSLocalizedString("Select location", comment: "Navigation title")
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
        TunnelManager.shared.reconnectTunnel {
            self.logger?.debug("Re-connected VPN tunnel")
        }
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

                    let alertController = UIAlertController(
                        title: NSLocalizedString("Failed to start the VPN tunnel", comment: ""),
                        message: error.errorChainDescription,
                        preferredStyle: .alert
                    )
                    alertController.addAction(
                        UIAlertAction(title: NSLocalizedString("OK", comment: ""), style: .cancel)
                    )

                    self.alertPresenter.enqueue(alertController, presentingController: self.rootContainer!)
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

                let alertController = UIAlertController(
                    title: NSLocalizedString("Failed to stop the VPN tunnel", comment: ""),
                    message: error.errorChainDescription,
                    preferredStyle: .alert
                )
                alertController.addAction(
                    UIAlertAction(title: NSLocalizedString("OK", comment: ""), style: .cancel)
                )

                self.alertPresenter.enqueue(alertController, presentingController: self.rootContainer!)
            }
        }
    }
}

// MARK: - SelectLocationViewControllerDelegate

extension AppDelegate: SelectLocationViewControllerDelegate {
    func selectLocationViewController(_ controller: SelectLocationViewController, didSelectRelayLocation relayLocation: RelayLocation) {
        self.window?.isUserInteractionEnabled = false
        DispatchQueue.main.asyncAfter(deadline: .now() + .milliseconds(250)) {
            self.window?.isUserInteractionEnabled = true
            controller.dismiss(animated: true) {
                self.selectLocationControllerDidSelectRelayLocation(relayLocation)
            }
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
