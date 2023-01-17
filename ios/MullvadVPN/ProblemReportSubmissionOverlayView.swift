//
//  ProblemReportSubmissionOverlayView.swift
//  MullvadVPN
//
//  Created by pronebird on 12/02/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import UIKit

class ProblemReportSubmissionOverlayView: UIView {
    var editButtonAction: (() -> Void)?
    var retryButtonAction: (() -> Void)?

    enum State {
        case sending
        case sent(_ email: String)
        case failure(REST.Error)

        var title: String? {
            switch self {
            case .sending:
                return NSLocalizedString(
                    "SUBMISSION_STATUS_SENDING",
                    tableName: "ProblemReport",
                    value: "Sending...",
                    comment: ""
                )
            case .sent:
                return NSLocalizedString(
                    "SUBMISSION_STATUS_SENT",
                    tableName: "ProblemReport",
                    value: "Sent",
                    comment: ""
                )
            case .failure:
                return NSLocalizedString(
                    "SUBMISSION_STATUS_FAILURE",
                    tableName: "ProblemReport",
                    value: "Failed to send",
                    comment: ""
                )
            }
        }

        var body: NSAttributedString? {
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
                    ),
                    attributes: [.foregroundColor: UIColor.successColor]
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

                return combinedAttributedString

            case let .failure(error):
                return error.displayErrorDescription.flatMap { NSAttributedString(string: $0) }
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
        indicator.tintColor = .white
        return indicator
    }()

    let statusImageView = StatusImageView(style: .success)

    let titleLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.font = UIFont.systemFont(ofSize: 32)
        textLabel.textColor = .white
        textLabel.numberOfLines = 0
        return textLabel
    }()

    let bodyLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.font = UIFont.systemFont(ofSize: 17)
        textLabel.textColor = .white
        textLabel.numberOfLines = 0
        return textLabel
    }()

    /// Footer stack view that contains action buttons
    private lazy var buttonsStackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [self.editMessageButton, self.tryAgainButton])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = 18

        return stackView
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

        addSubviews()
        transitionToState(state)

        layoutMargins = UIMetrics.contentLayoutMargins
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func addSubviews() {
        for subview in [
            titleLabel,
            bodyLabel,
            activityIndicator,
            statusImageView,
            buttonsStackView,
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

            bodyLabel.topAnchor.constraint(
                equalToSystemSpacingBelow: titleLabel.bottomAnchor,
                multiplier: 1
            ),
            bodyLabel.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor),
            bodyLabel.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor),
            buttonsStackView.topAnchor.constraint(
                greaterThanOrEqualTo: bodyLabel.bottomAnchor,
                constant: 18
            ),

            buttonsStackView.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor),
            buttonsStackView.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor),
            buttonsStackView.bottomAnchor.constraint(equalTo: layoutMarginsGuide.bottomAnchor),
        ])
    }

    private func transitionToState(_ state: State) {
        titleLabel.text = state.title
        bodyLabel.attributedText = state.body

        switch state {
        case .sending:
            activityIndicator.startAnimating()
            statusImageView.isHidden = true
            buttonsStackView.isHidden = true

        case .sent:
            activityIndicator.stopAnimating()
            statusImageView.style = .success
            statusImageView.isHidden = false
            buttonsStackView.isHidden = true

        case .failure:
            activityIndicator.stopAnimating()
            statusImageView.style = .failure
            statusImageView.isHidden = false
            buttonsStackView.isHidden = false
        }
    }

    // MARK: - Actions

    @objc private func handleEditButton() {
        editButtonAction?()
    }

    @objc private func handleRetryButton() {
        retryButtonAction?()
    }
}
