//
//  AppDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 19/03/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Combine
import UIKit

@UIApplicationMain
class AppDelegate: UIResponder, UIApplicationDelegate {

    var window: UIWindow?

    let mainStoryboard = UIStoryboard(name: "Main", bundle: nil)

    private var loadTunnelSubscriber: AnyCancellable?

    func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?) -> Bool {
        let accountToken = Account.shared.token

        loadTunnelSubscriber = TunnelManager.shared.loadTunnel(accountToken: accountToken)
            .receive(on: DispatchQueue.main)
            .sink(receiveCompletion: { (completion) in
                if case .failure(let error) = completion {
                    fatalError("Failed to restore the account: \(error.localizedDescription)")
                }

                let rootViewController = self.mainStoryboard.instantiateViewController(identifier: ViewControllerIdentifier.root.rawValue) as! RootContainerViewController

                let loginViewController = self.mainStoryboard.instantiateViewController(withIdentifier: ViewControllerIdentifier.login.rawValue)

                var viewControllers = [loginViewController]

                if accountToken != nil {
                    let mainViewController = self.mainStoryboard.instantiateViewController(withIdentifier: ViewControllerIdentifier.main.rawValue)

                    viewControllers.append(mainViewController)
                }

                rootViewController.setViewControllers(viewControllers, animated: false)

                self.window?.rootViewController = rootViewController
            })

        return true
    }

    func applicationWillResignActive(_ application: UIApplication) {
        // Sent when the application is about to move from active to inactive state. This can occur for certain types of temporary interruptions (such as an incoming phone call or SMS message) or when the user quits the application and it begins the transition to the background state.
        // Use this method to pause ongoing tasks, disable timers, and invalidate graphics rendering callbacks. Games should use this method to pause the game.
    }

    func applicationDidEnterBackground(_ application: UIApplication) {
        // Use this method to release shared resources, save user data, invalidate timers, and store enough application state information to restore your application to its current state in case it is terminated later.
        // If your application supports background execution, this method is called instead of applicationWillTerminate: when the user quits.
    }

    func applicationWillEnterForeground(_ application: UIApplication) {
        // Called as part of the transition from the background to the active state; here you can undo many of the changes made on entering the background.
    }

    func applicationDidBecomeActive(_ application: UIApplication) {
        // Restart any tasks that were paused (or not yet started) while the application was inactive. If the application was previously in the background, optionally refresh the user interface.
    }

    func applicationWillTerminate(_ application: UIApplication) {
        // Called when the application is about to terminate. Save data if appropriate. See also applicationDidEnterBackground:.
    }

}

