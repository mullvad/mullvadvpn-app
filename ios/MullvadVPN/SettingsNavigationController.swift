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

        navigationBar.barStyle = .black
        navigationBar.tintColor = .white
        navigationBar.prefersLargeTitles = true

        // Update account expiry
        Account.shared.updateAccountExpiry()
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
        switch route {
        case .account:
            let controller = AccountViewController()
            controller.delegate = self
            pushViewController(controller, animated: animated)

        case .preferences:
            pushViewController(PreferencesViewController(), animated: animated)

        case .wireguardKeys:
            pushViewController(WireguardKeysViewController(), animated: animated)

        case .problemReport:
            pushViewController(ProblemReportViewController(), animated: animated)
        }
    }

    // MARK: - UIAdaptivePresentationControllerDelegate

    func presentationControllerDidDismiss(_ presentationController: UIPresentationController) {
        settingsDelegate?.settingsNavigationController(self, didFinishWithReason: .none)
    }
}
