//
//  AppDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 19/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Combine
import UIKit
import StoreKit

@UIApplicationMain
class AppDelegate: UIResponder, UIApplicationDelegate {

    var window: UIWindow?

    let mainStoryboard = UIStoryboard(name: "Main", bundle: nil)

    #if targetEnvironment(simulator)
    let simulatorTunnelProvider = SimulatorTunnelProviderHost()
    #endif

    private var loadTunnelSubscriber: AnyCancellable?
    private var refreshTunnelSubscriber: AnyCancellable?

    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        #if targetEnvironment(simulator)
        SimulatorTunnelProvider.shared.delegate = simulatorTunnelProvider
        #endif

        let accountToken = Account.shared.token

        loadTunnelSubscriber = TunnelManager.shared.loadTunnel(accountToken: accountToken)
            .receive(on: DispatchQueue.main)
            .sink(receiveCompletion: { (completion) in
                if case .failure(let error) = completion {
                    fatalError("Failed to restore the account: \(error.localizedDescription)")
                }

                let rootViewController = self.mainStoryboard.instantiateViewController(identifier: ViewControllerIdentifier.root.rawValue) as! RootContainerViewController

                let showMainController = { (_ animated: Bool) in
                    self.showMainController(in: rootViewController, animated: animated) {
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
            })

        return true
    }

    func applicationDidBecomeActive(_ application: UIApplication) {
        refreshTunnelSubscriber = TunnelManager.shared.refreshTunnelState()
            .sink(receiveCompletion: { (_) in
                // no-op
            })
    }

    private func didPresentTheMainController() {
        let paymentManager = AppStorePaymentManager.shared
        paymentManager.delegate = self

        paymentManager.startPaymentQueueMonitoring()
        Account.shared.startPaymentMonitoring(with: paymentManager)
    }

    private func showTermsOfService(in rootViewController: RootContainerViewController, completionHandler: @escaping () -> Void) {
        let consentViewController = self.mainStoryboard.instantiateViewController(withIdentifier: ViewControllerIdentifier.consent.rawValue) as! ConsentViewController

        consentViewController.completionHandler = completionHandler

        rootViewController.setViewControllers([consentViewController], animated: false)
    }

    private func showMainController(
        in rootViewController: RootContainerViewController,
        animated: Bool,
        completionHandler: @escaping () -> Void)
    {
        let loginViewController = self.mainStoryboard.instantiateViewController(withIdentifier: ViewControllerIdentifier.login.rawValue)

        var viewControllers = [loginViewController]

        if Account.shared.isLoggedIn {
            let mainViewController = self.mainStoryboard.instantiateViewController(withIdentifier: ViewControllerIdentifier.main.rawValue)

            viewControllers.append(mainViewController)
        }

        rootViewController.setViewControllers(viewControllers, animated: animated, completion: completionHandler)
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
