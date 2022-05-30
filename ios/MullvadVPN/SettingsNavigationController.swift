//
//  SettingsNavigationController.swift
//  MullvadVPN
//
//  Created by pronebird on 02/07/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

enum SettingsNavigationRoute {
    case account
    case preferences
    case wireguardKeys
    case problemReport
}

enum SettingsDismissReason {
    case none
    case userLoggedOut
}

protocol SettingsNavigationControllerDelegate: AnyObject {
    func settingsNavigationController(_ controller: SettingsNavigationController, didFinishWithReason reason: SettingsDismissReason)
}

class SettingsNavigationController: CustomNavigationController, SettingsViewControllerDelegate, AccountViewControllerDelegate, UIAdaptivePresentationControllerDelegate {

    weak var settingsDelegate: SettingsNavigationControllerDelegate?

    override var childForStatusBarStyle: UIViewController? {
        return topViewController
    }

    override var childForStatusBarHidden: UIViewController? {
        return topViewController
    }

    init() {
        super.init(navigationBarClass: CustomNavigationBar.self, toolbarClass: nil)

        let settingsController = SettingsViewController()
        settingsController.delegate = self

        pushViewController(settingsController, animated: false)
    }

    override init(nibName nibNameOrNil: String?, bundle nibBundleOrNil: Bundle?) {
        // This initializer exists to prevent crash on iOS 12.
        // See: https://stackoverflow.com/a/38335090/351305
        super.init(nibName: nibNameOrNil, bundle: nibBundleOrNil)
    }

    required init?(coder aDecoder: NSCoder) {
        super.init(coder: aDecoder)
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        navigationBar.prefersLargeTitles = true

        // Update account expiry
        TunnelManager.shared.updateAccountData()
    }

    // MARK: - SettingsViewControllerDelegate

    func settingsViewControllerDidFinish(_ controller: SettingsViewController) {
        self.settingsDelegate?.settingsNavigationController(self, didFinishWithReason: .none)
    }

    // MARK: - AccountViewControllerDelegate

    func accountViewControllerDidLogout(_ controller: AccountViewController) {
        self.settingsDelegate?.settingsNavigationController(self, didFinishWithReason: .userLoggedOut)
    }

    // MARK: - Navigation

    func navigate(to route: SettingsNavigationRoute, animated: Bool) {
        let nextViewController = makeViewController(for: route)
        if let rootController = self.viewControllers.first, viewControllers.count > 1 {
            setViewControllers([rootController, nextViewController], animated: animated)
        } else {
            pushViewController(nextViewController, animated: animated)
        }
    }

    private func makeViewController(for route: SettingsNavigationRoute) -> UIViewController {
        switch route {
        case .account:
            let controller = AccountViewController()
            controller.delegate = self
            return controller

        case .preferences:
            return PreferencesViewController()

        case .wireguardKeys:
            return WireguardKeysViewController()

        case .problemReport:
            return ProblemReportViewController()
        }
    }

    // MARK: - UIAdaptivePresentationControllerDelegate

    func presentationControllerDidDismiss(_ presentationController: UIPresentationController) {
        settingsDelegate?.settingsNavigationController(self, didFinishWithReason: .none)
    }
}
