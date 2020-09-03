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
    var rootContainer: RootContainerViewController?

    #if targetEnvironment(simulator)
    let simulatorTunnelProvider = SimulatorTunnelProviderHost()
    #endif

    #if DEBUG
    private let packetTunnelLogForwarder = LogStreamer<UTF8>(fileURLs: [ApplicationConfiguration.packetTunnelLogFileURL!])
    #endif

    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        // Setup logging
        initLoggingSystem(bundleIdentifier: Bundle.main.bundleIdentifier!)

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
        RelayCache.shared.updateRelays()

        // Load tunnels
        let accountToken = Account.shared.token
        TunnelManager.shared.loadTunnel(accountToken: accountToken) { (result) in
            DispatchQueue.main.async {
                if case .failure(let error) = result {
                    fatalError(error.displayChain(message: "Failed to load the tunnel for account"))
                }

                let rootViewController = RootContainerViewController()
                rootViewController.delegate = self

                let showMainController = { (_ animated: Bool) in
                    self.showConnectController(in: rootViewController, animated: animated) {
                        self.didPresentTheMainController()
                    }
                }

                if Account.shared.isAgreedToTermsOfService {
                    showMainController(false)
                } else {
                    self.showTermsOfService(in: rootViewController) {
                        Account.shared.agreeToTermsOfService()

                        showMainController(true)
                    }
                }

                self.window?.rootViewController = rootViewController
                self.rootContainer = rootViewController
            }
        }

        // Show the window
        self.window?.makeKeyAndVisible()

        return true
    }

    func applicationDidBecomeActive(_ application: UIApplication) {
        TunnelManager.shared.refreshTunnelState(completionHandler: nil)
    }

    private func didPresentTheMainController() {
        let paymentManager = AppStorePaymentManager.shared
        paymentManager.delegate = self

        paymentManager.startPaymentQueueMonitoring()
        Account.shared.startPaymentMonitoring(with: paymentManager)
    }

    private func showTermsOfService(in rootViewController: RootContainerViewController, completionHandler: @escaping () -> Void) {
        let consentViewController = ConsentViewController()
        consentViewController.completionHandler = completionHandler

        rootViewController.setViewControllers([consentViewController], animated: false)
    }

    private func showConnectController(
        in rootViewController: RootContainerViewController,
        animated: Bool,
        completionHandler: @escaping () -> Void)
    {
        let loginViewController = LoginViewController()
        loginViewController.delegate = self

        var viewControllers: [UIViewController] = [loginViewController]

        if Account.shared.isLoggedIn {
            viewControllers.append(ConnectViewController())
        }

        rootViewController.setViewControllers(viewControllers, animated: animated, completion: completionHandler)
    }

}

extension AppDelegate: RootContainerViewControllerDelegate {

    func rootContainerViewControllerShouldShowSettings(_ controller: RootContainerViewController, navigateTo route: SettingsNavigationRoute?, animated: Bool) {
        let settingsController = SettingsViewController(style: .grouped)
        settingsController.settingsDelegate = self

        let navController = SettingsNavigationController(navigationBarClass: CustomNavigationBar.self, toolbarClass: nil)
        navController.pushViewController(settingsController, animated: false)

        if let route = route {
            settingsController.navigate(to: route)
        }

        controller.present(navController, animated: animated)
    }
}

extension AppDelegate: LoginViewControllerDelegate {

    func loginViewControllerDidLogin(_ controller: LoginViewController) {
        rootContainer?.pushViewController(ConnectViewController(), animated: true)
    }

}

extension AppDelegate: SettingsViewControllerDelegate {

    func settingsViewController(_ controller: SettingsViewController, didFinishWithReason reason: SettingsDismissReason) {
        if case .userLoggedOut = reason {
            rootContainer?.popToRootViewController(animated: false)

            let loginController = rootContainer?.topViewController as? LoginViewController

            loginController?.reset()
        }
        controller.dismiss(animated: true)
    }

}

extension AppDelegate: AppStorePaymentManagerDelegate {

    func appStorePaymentManager(_ manager: AppStorePaymentManager,
                                didRequestAccountTokenFor payment: SKPayment) -> String?
    {
        // Since we do not persist the relation between the payment and account token between the
        // app launches, we assume that all successful purchases belong to the active account token.
        return Account.shared.token
    }
}
