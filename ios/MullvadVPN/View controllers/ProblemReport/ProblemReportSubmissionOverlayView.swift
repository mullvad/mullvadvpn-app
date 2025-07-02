//
//  ProblemReportSubmissionOverlayView.swift
//  MullvadVPN
//
//  Created by pronebird on 12/02/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import UIKit

class ProblemReportSubmissionOverlayView: UIView {
    var viewLogsButtonAction: (() -> Void)?
    var cancelButtonAction: (() -> Void)?
    var editButtonAction: (() -> Void)?
    var retryButtonAction: (() -> Void)?

    enum State {
        case sending
        case sent(_ email: String)
        case failure(Error)

        var supportEmail: String {
            "support@mullvadvpn.net"
        }

        var title: String? {
            switch self {
            case .sending:
                NSLocalizedString(
                    "SUBMISSION_STATUS_SENDING",
                    tableName: "ProblemReport",
                    value: "Sending...",
                    comment: ""
                )
            case .sent:
                NSLocalizedString(
                    "SUBMISSION_STATUS_SENT",
                    tableName: "ProblemReport",
                    value: "Sent",
                    comment: ""
                )
            case .failure:
                NSLocalizedString(
                    "SUBMISSION_STATUS_FAILURE",
                    tableName: "ProblemReport",
                    value: "Failed to send",
                    comment: ""
                )
            }
        }

        var body: [NSAttributedString]? {
            switch self {
            case .sending:
                return nil
            case let .sent(email):
                let combinedAttributedString = NSMutableAttributedString(
                    string: NSLocalizedString(
                        "THANKS_MESSAGE",
                        tableName: "ProblemReport",
                        value: "Thanks!",
                        comment: ""
                    )
                )

                if email.isEmpty {
                    combinedAttributedString.append(NSAttributedString(string: " "))
                    combinedAttributedString.append(
                        NSAttributedString(
                            string: NSLocalizedString(
                                "WE_WILL_LOOK_INTO_THIS_MESSAGE",
                                tableName: "ProblemReport",
                                value: "We will look into this.",
                                comment: ""
                            )
                        )
                    )
                } else {
                    let emailText = String(
                        format: NSLocalizedString(
                            "CONTACT_BACK_EMAIL_MESSAGE_FORMAT",
                            tableName: "ProblemReport",
                            value: "If needed we will contact you at %@",
                            comment: ""
                        ), email
                    )
                    let emailAttributedString = NSMutableAttributedString(string: emailText)
                    if let emailRange = emailText.range(of: email) {
                        let font = UIFont.systemFont(ofSize: 17, weight: .bold)
                        let nsRange = NSRange(emailRange, in: emailText)

                        emailAttributedString.addAttribute(.font, value: font, range: nsRange)
                    }

                    combinedAttributedString.append(NSAttributedString(string: " "))
                    combinedAttributedString.append(emailAttributedString)
                }

                return [combinedAttributedString]

            case .failure:
                return [
                    NSAttributedString(
                        string: NSLocalizedString(
                            "MESSAGE_FAILED_PART_1",
                            tableName: "ProblemReport",
                            value:
                            """
                            If you exit the form and try again later, the information you already entered will still \
                            be here.
                            """,
                            comment: ""
                        )
                    ),
                    NSAttributedString(
                        markdownString: NSLocalizedString(
                            "MESSAGE_FAILED_PART_2",
                            tableName: "ProblemReport",
                            value:
                            """
                            If you still experience issues you can email our support directly at \
                            **\(supportEmail)**. Please attach your app log to your email.
                            """,
                            comment: ""
                        ),
                        options: MarkdownStylingOptions(
                            font: .preferredFont(forTextStyle: .body)
                        ), applyEffect: { _, _ in
                            [
                                // Setting font again to circumvent bold weight.
                                .font: UIFont.preferredFont(forTextStyle: .body),
                                .foregroundColor: UIColor.white,
                            ]
                        }
                    ),
                ]
            }
        }
    }

    var state: State = .sending {
        didSet {
            transitionToState(state)
        }
    }

    let activityIndicator: SpinnerActivityIndicatorView = {
        let indicator = SpinnerActivityIndicatorView(style: .large)
        return indicator
    }()

    let statusImageView = StatusImageView(style: .success)

    let titleLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.font = .mullvadLarge
        textLabel.adjustsFontForContentSizeCategory = true
        textLabel.textColor = .white
        textLabel.numberOfLines = 0
        return textLabel
    }()

    let bodyLabelContainer: UIStackView = {
        let stackView = UIStackView()
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = 24
        return stackView
    }()

    /// Footer stack view that contains action buttons.
    private lazy var buttonContainer: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [cancelButton, failedToSendButtons])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = 18
        return stackView
    }()

    /// Footer stack view that contains action buttons when sending failed.
    private lazy var failedToSendButtons: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [editMessageButton, viewLogsButton, tryAgainButton])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = 18
        return stackView
    }()

    private lazy var viewLogsButton: AppButton = {
        let button = AppButton(style: .default)
        button.setAccessibilityIdentifier(.problemReportAppLogsButton)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(ProblemReportViewModel.viewLogsButtonTitle, for: .normal)
        button.addTarget(self, action: #selector(handleViewLogsButton), for: .touchUpInside)
        return button
    }()

    private lazy var cancelButton: AppButton = {
        let button = AppButton(style: .default)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(NSLocalizedString(
            "CANCEL_BUTTON",
            tableName: "ProblemReport",
            value: "Cancel",
            comment: ""
        ), for: .normal)
        button.addTarget(self, action: #selector(handleCancelButton), for: .touchUpInside)
        return button
    }()

    private lazy var editMessageButton: AppButton = {
        let button = AppButton(style: .default)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(NSLocalizedString(
            "EDIT_MESSAGE_BUTTON",
            tableName: "ProblemReport",
            value: "Edit message",
            comment: ""
        ), for: .normal)
        button.addTarget(self, action: #selector(handleEditButton), for: .touchUpInside)
        return button
    }()

    private lazy var tryAgainButton: AppButton = {
        let button = AppButton(style: .success)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(NSLocalizedString(
            "TRY_AGAIN_BUTTON",
            tableName: "ProblemReport",
            value: "Try again",
            comment: ""
        ), for: .normal)
        button.addTarget(self, action: #selector(handleRetryButton), for: .touchUpInside)
        return button
    }()

    override init(frame: CGRect) {
        super.init(frame: frame)

        setAccessibilityIdentifier(.problemReportSubmittedView)

        addSubviews()
        transitionToState(state)

        directionalLayoutMargins = UIMetrics.contentLayoutMargins
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func addSubviews() {
        for subview in [
            titleLabel,
            bodyLabelContainer,
            activityIndicator,
            statusImageView,
            buttonContainer,
        ] {
            subview.translatesAutoresizingMaskIntoConstraints = false
            addSubview(subview)
        }

        NSLayoutConstraint.activate([
            statusImageView.topAnchor.constraint(
                equalTo: layoutMarginsGuide.topAnchor,
                constant: 32
            ),
            statusImageView.centerXAnchor.constraint(equalTo: centerXAnchor),

            activityIndicator.centerXAnchor.constraint(equalTo: statusImageView.centerXAnchor),
            activityIndicator.centerYAnchor.constraint(equalTo: statusImageView.centerYAnchor),

            titleLabel.topAnchor.constraint(equalTo: statusImageView.bottomAnchor, constant: 60),
            titleLabel.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor),
            titleLabel.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor),

            bodyLabelContainer.topAnchor.constraint(
                equalToSystemSpacingBelow: titleLabel.bottomAnchor,
                multiplier: 1
            ),
            bodyLabelContainer.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor),
            bodyLabelContainer.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor),
            buttonContainer.topAnchor.constraint(
                greaterThanOrEqualTo: bodyLabelContainer.bottomAnchor,
                constant: 18
            ),

            buttonContainer.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor),
            buttonContainer.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor),
            buttonContainer.bottomAnchor.constraint(equalTo: layoutMarginsGuide.bottomAnchor),
        ])
    }

    private func transitionToState(_ state: State) {
        titleLabel.text = state.title

        bodyLabelContainer.subviews.forEach { $0.removeFromSuperview() }
        state.body?.forEach { attributedString in
            let textLabel = UILabel()
            textLabel.font = .mullvadSmall
            textLabel.adjustsFontForContentSizeCategory = true
            textLabel.textColor = .white.withAlphaComponent(0.6)
            textLabel.numberOfLines = 0
            textLabel.attributedText = attributedString

            if attributedString.string.contains(state.supportEmail) {
                let tapGesture = UITapGestureRecognizer(target: self, action: #selector(handleEmailLabelTap))
                textLabel.addGestureRecognizer(tapGesture)
                textLabel.isUserInteractionEnabled = true
            }

            bodyLabelContainer.addArrangedSubview(textLabel)
        }

        switch state {
        case .sending:
            activityIndicator.startAnimating()
            statusImageView.isHidden = true
            cancelButton.isHidden = false
            failedToSendButtons.isHidden = true

        case .sent:
            activityIndicator.stopAnimating()
            statusImageView.style = .success
            statusImageView.isHidden = false
            buttonContainer.isHidden = true

        case .failure:
            activityIndicator.stopAnimating()
            statusImageView.style = .failure
            statusImageView.isHidden = false
            cancelButton.isHidden = true
            failedToSendButtons.isHidden = false
        }
    }

    // MARK: - Actions

    @objc private func handleEmailLabelTap() {
        if let url = URL(string: "mailto:\(state.supportEmail)") {
            UIApplication.shared.open(url)
        }
    }

    @objc private func handleViewLogsButton() {
        viewLogsButtonAction?()
    }

    @objc private func handleCancelButton() {
        cancelButtonAction?()
    }

    @objc private func handleEditButton() {
        editButtonAction?()
    }

    @objc private func handleRetryButton() {
        retryButtonAction?()
    }
}
