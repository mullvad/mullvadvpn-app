//
//  RevokedDeviceViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 07/07/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import UIKit

protocol RevokedDeviceViewControllerDelegate: AnyObject {
    func revokedDeviceControllerDidRequestLogout(_ controller: RevokedDeviceViewController)
}

class RevokedDeviceViewController: UIViewController, RootContainment {
    private lazy var imageView: StatusImageView = {
        let statusImageView = StatusImageView(style: .failure)
        statusImageView.translatesAutoresizingMaskIntoConstraints = false
        return statusImageView
    }()

    private lazy var titleLabel: UILabel = {
        let titleLabel = UILabel()
        titleLabel.translatesAutoresizingMaskIntoConstraints = false
        titleLabel.font = UIFont.systemFont(ofSize: 24, weight: .bold)
        titleLabel.numberOfLines = 0
        titleLabel.textColor = .white
        titleLabel.text = NSLocalizedString(
            "TITLE_LABEL",
            tableName: "RevokedDevice",
            value: "Device is inactive",
            comment: ""
        )
        titleLabel.font = UIFont.systemFont(ofSize: 32)
        return titleLabel
    }()

    private lazy var bodyLabel: UILabel = {
        let bodyLabel = UILabel()
        bodyLabel.translatesAutoresizingMaskIntoConstraints = false
        bodyLabel.font = UIFont.systemFont(ofSize: 17)
        bodyLabel.numberOfLines = 0
        bodyLabel.textColor = .white
        bodyLabel.text = NSLocalizedString(
            "DESCRIPTION_LABEL",
            tableName: "RevokedDevice",
            value: "You have revoked this device. To connect again, you will need to log back in.",
            comment: ""
        )
        return bodyLabel
    }()

    private lazy var footerLabel: UILabel = {
        let bodyLabel = UILabel()
        bodyLabel.translatesAutoresizingMaskIntoConstraints = false
        bodyLabel.font = UIFont.systemFont(ofSize: 17)
        bodyLabel.numberOfLines = 0
        bodyLabel.textColor = .white
        bodyLabel.text = NSLocalizedString(
            "UNBLOCK_INTERNET_LABEL",
            tableName: "RevokedDevice",
            value: "Going to login will unblock the Internet on this device.",
            comment: ""
        )
        return bodyLabel
    }()

    private lazy var logoutButton: AppButton = {
        let button = AppButton(style: .default)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(
            NSLocalizedString(
                "GOTO_LOGIN_BUTTON_LABEL",
                tableName: "RevokedDevice",
                value: "Go to login",
                comment: ""
            ),
            for: .normal
        )
        return button
    }()

    weak var delegate: RevokedDeviceViewControllerDelegate?

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    var preferredHeaderBarPresentation: HeaderBarPresentation {
        let tunnelState = interactor.tunnelStatus.state

        return HeaderBarPresentation(
            style: tunnelState.isSecured ? .secured : .unsecured,
            showsDivider: true
        )
    }

    var prefersHeaderBarHidden: Bool {
        return false
    }

    private let interactor: RevokedDeviceInteractor

    init(interactor: RevokedDeviceInteractor) {
        self.interactor = interactor
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.backgroundColor = .secondaryColor
        view.layoutMargins = UIMetrics.contentLayoutMargins

        for subview in [imageView, titleLabel, bodyLabel, footerLabel, logoutButton] {
            view.addSubview(subview)
        }

        logoutButton.addTarget(
            self,
            action: #selector(didTapLogoutButton(_:)),
            for: .touchUpInside
        )

        NSLayoutConstraint.activate([
            imageView.topAnchor.constraint(
                equalTo: view.layoutMarginsGuide.topAnchor,
                constant: 30
            ),
            imageView.centerXAnchor.constraint(equalTo: view.centerXAnchor),

            titleLabel.topAnchor.constraint(
                equalTo: imageView.bottomAnchor,
                constant: 30
            ),
            titleLabel.leadingAnchor.constraint(equalTo: view.layoutMarginsGuide.leadingAnchor),
            titleLabel.trailingAnchor.constraint(equalTo: view.layoutMarginsGuide.trailingAnchor),

            bodyLabel.topAnchor.constraint(
                equalTo: titleLabel.bottomAnchor,
                constant: 16
            ),
            bodyLabel.leadingAnchor.constraint(equalTo: view.layoutMarginsGuide.leadingAnchor),
            bodyLabel.trailingAnchor.constraint(equalTo: view.layoutMarginsGuide.trailingAnchor),

            footerLabel.topAnchor.constraint(
                equalTo: bodyLabel.bottomAnchor,
                constant: 16
            ),
            footerLabel.leadingAnchor.constraint(equalTo: view.layoutMarginsGuide.leadingAnchor),
            footerLabel.trailingAnchor.constraint(equalTo: view.layoutMarginsGuide.trailingAnchor),

            logoutButton.topAnchor.constraint(greaterThanOrEqualTo: footerLabel.bottomAnchor),
            logoutButton.leadingAnchor.constraint(equalTo: view.layoutMarginsGuide.leadingAnchor),
            logoutButton.trailingAnchor.constraint(equalTo: view.layoutMarginsGuide.trailingAnchor),
            logoutButton.bottomAnchor.constraint(equalTo: view.layoutMarginsGuide.bottomAnchor),
        ])

        interactor.didUpdateTunnelStatus = { [weak self] tunnelStatus in
            self?.setNeedsHeaderBarStyleAppearanceUpdate()
            self?.updateView(tunnelState: tunnelStatus.state)
        }

        updateView(tunnelState: interactor.tunnelStatus.state)
    }

    @objc private func didTapLogoutButton(_ sender: Any?) {
        logoutButton.isEnabled = false

        delegate?.revokedDeviceControllerDidRequestLogout(self)
    }

    private func updateView(tunnelState: TunnelState) {
        logoutButton.style = tunnelState.isSecured ? .danger : .default
        footerLabel.isHidden = !tunnelState.isSecured
    }
}
