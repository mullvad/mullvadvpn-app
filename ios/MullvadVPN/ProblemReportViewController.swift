//
//  ProblemReportViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 15/09/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import UIKit

class ProblemReportViewController: UIViewController {

    let scrollView = UIScrollView()
    let containerView = UIView()

    let subheaderLabel = UILabel()

    let emailTextField = CustomTextField()
    let descriptionTextView = CustomTextView()
    let formContainerView = UIView()

    let actionFooter = UIStackView()
    let viewLogsButton = AppButton(style: .default)
    let sendButton = AppButton(style: .success)

    override func viewDidLoad() {
        super.viewDidLoad()

        view.backgroundColor = .secondaryColor
        containerView.layoutMargins = UIEdgeInsets(top: 12, left: 12, bottom: 12, right: 12)
        containerView.backgroundColor = .clear
        scrollView.backgroundColor = .clear

        navigationItem.title = NSLocalizedString("Report a problem", comment: "Navigation title")
        if #available(iOS 13.0, *) {
            isModalInPresentation = true
        }

        subheaderLabel.numberOfLines = 0
        subheaderLabel.textColor = .white
        subheaderLabel.text = NSLocalizedString("To help you more effectively, your app's log file will be attached to this message. Your data will remain secure and private, as it is anonymised before being sent over an encrypted channel.", comment: "")

        emailTextField.borderStyle = .none
        emailTextField.backgroundColor = .white
        emailTextField.font = UIFont.systemFont(ofSize: 17)
        emailTextField.placeholder = NSLocalizedString("Your email (optional)", comment: "")
        emailTextField.setContentHuggingPriority(.defaultHigh, for: .vertical)
        emailTextField.setContentCompressionResistancePriority(.defaultHigh, for: .vertical)

        descriptionTextView.backgroundColor = .white
        descriptionTextView.font = UIFont.systemFont(ofSize: 17)
        descriptionTextView.placeholder = NSLocalizedString("Describe your problem", comment: "")
        descriptionTextView.font = UIFont.systemFont(ofSize: 17)
        descriptionTextView.setContentHuggingPriority(.defaultLow, for: .vertical)
        descriptionTextView.setContentCompressionResistancePriority(.defaultLow, for: .vertical)

        formContainerView.addSubview(emailTextField)
        formContainerView.addSubview(descriptionTextView)

        subheaderLabel.translatesAutoresizingMaskIntoConstraints = false
        scrollView.translatesAutoresizingMaskIntoConstraints = false
        containerView.translatesAutoresizingMaskIntoConstraints = false
        formContainerView.translatesAutoresizingMaskIntoConstraints = false
        emailTextField.translatesAutoresizingMaskIntoConstraints = false
        descriptionTextView.translatesAutoresizingMaskIntoConstraints = false
        actionFooter.translatesAutoresizingMaskIntoConstraints = false

        actionFooter.axis = .vertical
        actionFooter.spacing = 12
        actionFooter.addArrangedSubview(viewLogsButton)
        actionFooter.addArrangedSubview(sendButton)

        sendButton.setTitle(NSLocalizedString("Send", comment: ""), for: .normal)
        viewLogsButton.setTitle(NSLocalizedString("View app logs", comment: ""), for: .normal)

        view.addSubview(scrollView)
        scrollView.addSubview(containerView)
        containerView.addSubview(subheaderLabel)
        containerView.addSubview(formContainerView)
        containerView.addSubview(actionFooter)

        let constraints = [
            subheaderLabel.topAnchor.constraint(equalTo: containerView.layoutMarginsGuide.topAnchor),
            subheaderLabel.leadingAnchor.constraint(equalTo: containerView.layoutMarginsGuide.leadingAnchor),
            subheaderLabel.trailingAnchor.constraint(equalTo: containerView.layoutMarginsGuide.trailingAnchor),

            formContainerView.topAnchor.constraint(equalTo: subheaderLabel.bottomAnchor, constant: 12),
            formContainerView.leadingAnchor.constraint(equalTo: containerView.layoutMarginsGuide.leadingAnchor),
            formContainerView.trailingAnchor.constraint(equalTo: containerView.layoutMarginsGuide.trailingAnchor),

            actionFooter.topAnchor.constraint(equalTo: formContainerView.bottomAnchor, constant: 12),
            actionFooter.leadingAnchor.constraint(equalTo: containerView.layoutMarginsGuide.leadingAnchor),
            actionFooter.trailingAnchor.constraint(equalTo: containerView.layoutMarginsGuide.trailingAnchor),
            actionFooter.bottomAnchor.constraint(equalTo: containerView.layoutMarginsGuide.bottomAnchor),

            emailTextField.topAnchor.constraint(equalTo: formContainerView.topAnchor),
            emailTextField.leadingAnchor.constraint(equalTo: formContainerView.leadingAnchor),
            emailTextField.trailingAnchor.constraint(equalTo: formContainerView.trailingAnchor),

            descriptionTextView.topAnchor.constraint(equalTo: emailTextField.bottomAnchor, constant: 12),
            descriptionTextView.leadingAnchor.constraint(equalTo: formContainerView.leadingAnchor),
            descriptionTextView.trailingAnchor.constraint(equalTo: formContainerView.trailingAnchor),
            descriptionTextView.bottomAnchor.constraint(equalTo: formContainerView.bottomAnchor),

            scrollView.frameLayoutGuide.topAnchor.constraint(equalTo: view.topAnchor),
            scrollView.frameLayoutGuide.bottomAnchor.constraint(equalTo: view.bottomAnchor),
            scrollView.frameLayoutGuide.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            scrollView.frameLayoutGuide.trailingAnchor.constraint(equalTo: view.trailingAnchor),

            scrollView.contentLayoutGuide.topAnchor.constraint(equalTo: containerView.topAnchor),
            scrollView.contentLayoutGuide.bottomAnchor.constraint(equalTo: containerView.bottomAnchor),
            scrollView.contentLayoutGuide.leadingAnchor.constraint(equalTo: containerView.leadingAnchor),
            scrollView.contentLayoutGuide.trailingAnchor.constraint(equalTo: containerView.trailingAnchor),

            scrollView.contentLayoutGuide.widthAnchor.constraint(equalTo: scrollView.frameLayoutGuide.widthAnchor),
            scrollView.contentLayoutGuide.heightAnchor.constraint(greaterThanOrEqualTo: scrollView.safeAreaLayoutGuide.heightAnchor),

            descriptionTextView.heightAnchor.constraint(greaterThanOrEqualToConstant: 150),
        ]

        NSLayoutConstraint.activate(constraints)
    }

}
