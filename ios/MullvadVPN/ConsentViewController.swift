//
//  ConsentViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 21/02/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import SafariServices
import UIKit

class ConsentViewController: UIViewController, RootContainment, SFSafariViewControllerDelegate {

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    var preferredHeaderBarStyle: HeaderBarStyle {
        return .transparent
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        self.rootContainerController?.headerBarSettingsButton.isHidden = true
    }

    // MARK: - IBActions

    @IBAction func openPrivacyPolicy(_ sender: Any) {
        let pageURL = URL(string: "https://mullvad.net/en/help/privacy-policy/?hide_nav")!

        let safariController = SFSafariViewController(url: pageURL)
        safariController.delegate = self

        let navigationController = UINavigationController(rootViewController: safariController)
        navigationController.setNavigationBarHidden(true, animated: false)

        present(navigationController, animated: true)
    }

    // MARK: - SFSafariViewControllerDelegate

    func safariViewControllerDidFinish(_ controller: SFSafariViewController) {
        controller.dismiss(animated: true)
    }

}
