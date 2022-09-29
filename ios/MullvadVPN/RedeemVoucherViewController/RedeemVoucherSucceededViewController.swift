//
//  RedeemVoucherSucceededViewController.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-09-23.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import UIKit

protocol RedeemVoucherSucceededViewControllerDelegate: AnyObject {
    func redeemVoucherSucceededViewControllerDidFinish(
        _ controller: RedeemVoucherSucceededViewController
    )
}

class RedeemVoucherSucceededViewController: UIViewController {
    private let statusImageView: StatusImageView = {
        let statusImageView = StatusImageView(style: .success)
        statusImageView.translatesAutoresizingMaskIntoConstraints = false
        return statusImageView
    }()

    private let titleLabel: UILabel = {
        let label = UILabel()
        label.font = UIFont.boldSystemFont(ofSize: 20)
        label.text = NSLocalizedString(
            "REDEEM_VOUCHER_SUCCESS_TITLE",
            tableName: "RedeemVoucher",
            value: "Voucher was successfully redeemed.",
            comment: ""
        )
        label.textColor = .white
        label.numberOfLines = 0
        label.translatesAutoresizingMaskIntoConstraints = false
        return label
    }()

    private let messageLabel: UILabel = {
        let label = UILabel()
        label.font = UIFont.systemFont(ofSize: 17)
        label.textColor = UIColor.white.withAlphaComponent(0.6)
        label.numberOfLines = 0
        label.translatesAutoresizingMaskIntoConstraints = false
        return label
    }()

    private let dismissButton: AppButton = {
        let button = AppButton(style: .default)
        button.setTitle(NSLocalizedString(
            "REDEEM_VOUCHER_DISMISS_BUTTON",
            tableName: "RedeemVoucher",
            value: "Got it!",
            comment: ""
        ), for: .normal)
        button.translatesAutoresizingMaskIntoConstraints = false
        return button
    }()

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    weak var delegate: RedeemVoucherSucceededViewControllerDelegate?

    init(timeAddedComponents: DateComponents) {
        super.init(nibName: nil, bundle: nil)

        view.backgroundColor = .secondaryColor
        view.layoutMargins = UIMetrics.contentLayoutMargins

        messageLabel.text = String(
            format: NSLocalizedString(
                "REDEEM_VOUCHER_SUCCESS_MESSAGE",
                tableName: "RedeemVoucher",
                value: "%@ were added to your account.",
                comment: ""
            ),
            Self.formatTimeAdded(from: timeAddedComponents)
        )
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        addSubviews()
        addConstraints()
        addDismissButtonHandler()
    }

    private func addSubviews() {
        for subview in [statusImageView, titleLabel, messageLabel, dismissButton] {
            view.addSubview(subview)
        }
    }

    private func addConstraints() {
        NSLayoutConstraint.activate([
            statusImageView.topAnchor.constraint(equalTo: view.layoutMarginsGuide.topAnchor),
            statusImageView.centerXAnchor.constraint(equalTo: view.centerXAnchor),

            titleLabel.topAnchor.constraint(
                equalTo: statusImageView.bottomAnchor,
                constant: UIMetrics.sectionSpacing
            ),
            titleLabel.leadingAnchor
                .constraint(equalTo: view.layoutMarginsGuide.leadingAnchor),
            titleLabel.trailingAnchor
                .constraint(equalTo: view.layoutMarginsGuide.trailingAnchor),

            messageLabel.topAnchor.constraint(
                equalTo: titleLabel.layoutMarginsGuide.bottomAnchor,
                constant: UIMetrics.StackSpacing.regular
            ),
            messageLabel.leadingAnchor.constraint(equalTo: view.layoutMarginsGuide.leadingAnchor),
            messageLabel.trailingAnchor.constraint(equalTo: view.layoutMarginsGuide.trailingAnchor),

            dismissButton.bottomAnchor.constraint(equalTo: view.layoutMarginsGuide.bottomAnchor),
            dismissButton.leadingAnchor.constraint(equalTo: view.layoutMarginsGuide.leadingAnchor),
            dismissButton.trailingAnchor
                .constraint(equalTo: view.layoutMarginsGuide.trailingAnchor),
        ])
    }

    private func addDismissButtonHandler() {
        dismissButton.addTarget(
            self,
            action: #selector(handleDismissTap),
            for: .touchUpInside
        )
    }

    @objc private func handleDismissTap() {
        delegate?.redeemVoucherSucceededViewControllerDidFinish(self)
    }

    private static func formatTimeAdded(from timeAdded: DateComponents) -> String {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.day]
        formatter.unitsStyle = .full

        return formatter.string(from: timeAdded) ?? ""
    }
}
