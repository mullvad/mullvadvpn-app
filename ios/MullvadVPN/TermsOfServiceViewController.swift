//
//  TermsOfServiceViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 21/02/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import SafariServices
import UIKit

protocol TermsOfServiceViewControllerDelegate: AnyObject {
    func termsOfServiceViewControllerDidFinish(_ controller: TermsOfServiceViewController)
}

class TermsOfServiceViewController: UIViewController, RootContainment,
    SFSafariViewControllerDelegate
{
    weak var delegate: TermsOfServiceViewControllerDelegate?

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    var preferredHeaderBarPresentation: HeaderBarPresentation {
        return HeaderBarPresentation(style: .default, showsDivider: false)
    }

    var prefersHeaderBarHidden: Bool {
        return false
    }

    // MARK: - View lifecycle

    override func viewDidLoad() {
        super.viewDidLoad()

        let contentView = TermsOfServiceContentView()
        contentView.translatesAutoresizingMaskIntoConstraints = false
        contentView.agreeButton.addTarget(
            self,
            action: #selector(handleAgreeButton),
            for: .touchUpInside
        )
        contentView.privacyPolicyLink.addTarget(
            self,
            action: #selector(handlePrivacyPolicyButton),
            for: .touchUpInside
        )

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

    @objc private func handlePrivacyPolicyButton() {
        let safariController = SFSafariViewController(
            url: ApplicationConfiguration.privacyPolicyURL
        )
        safariController.delegate = self

        present(safariController, animated: true)
    }

    @objc private func handleAgreeButton() {
        delegate?.termsOfServiceViewControllerDidFinish(self)
    }

    // MARK: - SFSafariViewControllerDelegate

    func safariViewControllerDidFinish(_ controller: SFSafariViewController) {
        controller.dismiss(animated: true)
    }

    func safariViewControllerWillOpenInBrowser(_ controller: SFSafariViewController) {
        controller.dismiss(animated: false)
    }
}
