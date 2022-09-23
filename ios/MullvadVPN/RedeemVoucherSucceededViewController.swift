//
//  RedeemVoucherSucceededViewController.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-09-23.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import UIKit

class RedeemVoucherSucceededViewController: UIViewController {
    // MARK: - Views

    private let successImage: UIImageView = {
        let imageView = UIImageView(image: UIImage(named: "IconSuccess"))
        imageView.contentMode = .scaleAspectFit
        imageView.translatesAutoresizingMaskIntoConstraints = false
        return imageView
    }()

    private let instructionLabel: UILabel = {
        let label = UILabel()
        label.font = UIFont.boldSystemFont(ofSize: 20)
        label.text = NSLocalizedString(
            "REDEEM_VOUCHER_INSTRUCTION_SUCCESS",
            tableName: "RedeemVoucher",
            value: "Voucher was successfully redeemed.",
            comment: ""
        )
        label.textColor = .white
        label.numberOfLines = 0
        label.translatesAutoresizingMaskIntoConstraints = false
        return label
    }()

    private let statusLabel: UILabel = {
        let label = UILabel()
        label.font = UIFont.systemFont(ofSize: 17)
        label.textColor = UIColor.white.withAlphaComponent(0.6)
        label.numberOfLines = 0
        label.translatesAutoresizingMaskIntoConstraints = false
        return label
    }()

    private let gotItButton: AppButton = {
        let button = AppButton(style: .default)
        button.setTitle(NSLocalizedString(
            "REDEEM_VOUCHER_GOT_IT_BUTTON",
            tableName: "RedeemVoucher",
            value: "Got it!",
            comment: ""
        ), for: .normal)
        button.translatesAutoresizingMaskIntoConstraints = false
        return button
    }()

    override var preferredStatusBarStyle: UIStatusBarStyle { .lightContent }

    let timeAdded: String

    init(timeAdded: String) {
        self.timeAdded = timeAdded
        super.init(nibName: nil, bundle: nil)


        view.backgroundColor = .secondaryColor
        view.layoutMargins = UIMetrics.contentLayoutMargins

        statusLabel.text = "\(timeAdded) were added to your account."
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        addViews()
        constraintViews()
        subscribeClicks()
    }

    // MARK: - View setup

    private func addViews() {
        view.addSubview(successImage)
        view.addSubview(instructionLabel)
        view.addSubview(statusLabel)
        view.addSubview(gotItButton)
    }

    private func constraintViews() {
        NSLayoutConstraint.activate([
            successImage.topAnchor.constraint(equalTo: view.layoutMarginsGuide.topAnchor),
            successImage.centerXAnchor.constraint(equalTo: view.centerXAnchor),
            successImage.widthAnchor.constraint(equalToConstant: 60),
            successImage.heightAnchor.constraint(equalToConstant: 60)
        ])

        NSLayoutConstraint.activate([
            instructionLabel.topAnchor.constraint(equalTo: successImage.bottomAnchor, constant: 24),
            instructionLabel.leadingAnchor.constraint(equalTo: view.layoutMarginsGuide.leadingAnchor),
            instructionLabel.trailingAnchor.constraint(equalTo: view.layoutMarginsGuide.trailingAnchor),
        ])

        NSLayoutConstraint.activate([
            statusLabel.topAnchor.constraint(equalTo: instructionLabel.layoutMarginsGuide.bottomAnchor, constant: 12),
            statusLabel.leadingAnchor.constraint(equalTo: view.layoutMarginsGuide.leadingAnchor),
            statusLabel.trailingAnchor.constraint(equalTo: view.layoutMarginsGuide.trailingAnchor),
        ])

        NSLayoutConstraint.activate([
            gotItButton.bottomAnchor.constraint(equalTo: view.layoutMarginsGuide.bottomAnchor),
            gotItButton.leadingAnchor.constraint(equalTo: view.layoutMarginsGuide.leadingAnchor),
            gotItButton.trailingAnchor.constraint(equalTo: view.layoutMarginsGuide.trailingAnchor),
            gotItButton.heightAnchor.constraint(equalToConstant: 42)
        ])
    }

    private func subscribeClicks() {
        gotItButton.addTarget(self,
                              action: #selector(gotItButtonClicked),
                              for: .touchUpInside)
    }

    @objc private func gotItButtonClicked(_ sender: AppButton) {
        self.dismiss(animated: true)
    }
}
