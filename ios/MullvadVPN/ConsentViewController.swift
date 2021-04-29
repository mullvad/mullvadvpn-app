//
//  ConsentViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 21/02/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import SafariServices
import UIKit

private let kPrivacyPolicyURL = URL(string: "https://mullvad.net/en/help/privacy-policy/?hide_nav")!

class ConsentViewController: UIViewController, RootContainment, SFSafariViewControllerDelegate {

    var completionHandler: (() -> Void)?

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    var preferredHeaderBarStyle: HeaderBarStyle {
        return .transparent
    }

    var prefersHeaderBarHidden: Bool {
        return true
    }

    // MARK: - View lifecycle

    override func viewDidLoad() {
        super.viewDidLoad()

        let contentView = ConsentContentView()
        contentView.translatesAutoresizingMaskIntoConstraints = false
        contentView.agreeButton.addTarget(self, action: #selector(handleAgreeButton(_:)), for: .touchUpInside)
        contentView.privacyPolicyLink.addTarget(self, action: #selector(handlePrivacyPolicyButton(_:)), for: .touchUpInside)

        view.backgroundColor = .primaryColor
        view.addSubview(contentView)

        NSLayoutConstraint.activate([
            contentView.topAnchor.constraint(equalTo: view.topAnchor),
            contentView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            contentView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            contentView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])
    }

    // MARK: - Actions

    @objc private func handlePrivacyPolicyButton(_ sender: Any) {
        let safariController = SFSafariViewController(url: kPrivacyPolicyURL)
        safariController.delegate = self

        present(safariController, animated: true)
    }

    @objc private func handleAgreeButton(_ sender: Any) {
        completionHandler?(self)
    }

    // MARK: - SFSafariViewControllerDelegate

    func safariViewControllerDidFinish(_ controller: SFSafariViewController) {
        controller.dismiss(animated: true)
    }

}
