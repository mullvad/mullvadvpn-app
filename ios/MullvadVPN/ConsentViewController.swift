//
//  ConsentViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 21/02/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import SafariServices
import UIKit

let kPrivacyPolicyURL = URL(string: "https://mullvad.net/en/help/privacy-policy/?hide_nav")!

class ConsentViewController: UIViewController, RootContainment, SFSafariViewControllerDelegate {

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    var preferredHeaderBarStyle: HeaderBarStyle {
        return .transparent
    }

    var prefersHeaderBarHidden: Bool {
        return true
    }

    // MARK: - IBActions

    @IBAction func handlePrivacyPolicyButton(_ sender: Any) {
        let safariController = SFSafariViewController(url: kPrivacyPolicyURL)
        safariController.delegate = self

        present(safariController, animated: true)
    }

    // MARK: - SFSafariViewControllerDelegate

    func safariViewControllerDidFinish(_ controller: SFSafariViewController) {
        controller.dismiss(animated: true)
    }

}
