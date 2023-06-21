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
        .lightContent
    }

    weak var delegate: RedeemVoucherSucceededViewControllerDelegate?

    init(timeAddedComponents: DateComponents) {
        super.init(nibName: nil, bundle: nil)

        view.backgroundColor = .secondaryColor
        view.directionalLayoutMargins = UIMetrics.contentLayoutMargins

        messageLabel.text = String(
            format: NSLocalizedString(
                "REDEEM_VOUCHER_SUCCESS_MESSAGE",
                tableName: "RedeemVoucher",
                value: "%@ were added to your account.",
                comment: ""
            ),
            timeAddedComponents.formattedAddedDay
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
        view.addConstrainedSubviews([statusImageView, titleLabel, messageLabel, dismissButton]) {
            statusImageView.pinEdgesToSuperviewMargins(PinnableEdges([.top(0)]))
            statusImageView.centerXAnchor.constraint(equalTo: view.centerXAnchor)

            titleLabel.pinEdgesToSuperviewMargins(PinnableEdges([.leading(0), .trailing(0)]))
            titleLabel.topAnchor.constraint(
                equalTo: statusImageView.bottomAnchor,
                constant: UIMetrics.sectionSpacing
            )

            messageLabel.topAnchor.constraint(
                equalTo: titleLabel.layoutMarginsGuide.bottomAnchor,
                constant: UIMetrics.interButtonSpacing
            )
            messageLabel.pinEdgesToSuperviewMargins(PinnableEdges([.leading(0), .trailing(0)]))

            dismissButton.pinEdgesToSuperviewMargins(.all().excluding(.top))
        }
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
}

private extension DateComponents {
    var formattedAddedDay: String {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.day]
        formatter.unitsStyle = .full
        return formatter.string(from: self) ?? ""
    }
}
