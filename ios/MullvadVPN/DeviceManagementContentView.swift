//
//  DeviceManagementContentView.swift
//  MullvadVPN
//
//  Created by pronebird on 19/07/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import UIKit

class DeviceManagementContentView: UIView {
    private let scrollView: UIScrollView = {
        let scrollView = UIScrollView()
        scrollView.translatesAutoresizingMaskIntoConstraints = false
        return scrollView
    }()

    let contentView: UIView = {
        let view = UIView()
        view.layoutMargins = UIMetrics.contentLayoutMargins
        view.translatesAutoresizingMaskIntoConstraints = false
        return view
    }()

    let statusImageView: StatusImageView = {
        let imageView = StatusImageView(style: .failure)
        imageView.translatesAutoresizingMaskIntoConstraints = false
        return imageView
    }()

    let titleLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.font = UIFont.systemFont(ofSize: 32)
        textLabel.textColor = .white
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        return textLabel
    }()

    let messageLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.font = UIFont.systemFont(ofSize: 17)
        textLabel.textColor = .white
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.numberOfLines = 0
        if #available(iOS 14.0, *) {
            // See: https://stackoverflow.com/q/46200027/351305
            textLabel.lineBreakStrategy = []
        }
        return textLabel
    }()

    let deviceStackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.spacing = 1
        stackView.clipsToBounds = true
        stackView.distribution = .fillEqually
        return stackView
    }()

    let continueButton: AppButton = {
        let button = AppButton(style: .success)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(
            NSLocalizedString(
                "CONTINUE_BUTTON",
                tableName: "DeviceManagement",
                value: "Continue with login",
                comment: ""
            ),
            for: .normal
        )
        button.isEnabled = false
        return button
    }()

    let backButton: AppButton = {
        let button = AppButton(style: .default)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setTitle(
            NSLocalizedString(
                "BACK_BUTTON",
                tableName: "DeviceManagement",
                value: "Back",
                comment: ""
            ),
            for: .normal
        )
        return button
    }()

    private lazy var buttonStackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [continueButton, backButton])
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.axis = .vertical
        stackView.distribution = .fillEqually
        stackView.spacing = UIMetrics.interButtonSpacing
        return stackView
    }()

    var handleDeviceDeletion: ((DeviceViewModel, @escaping () -> Void) -> Void)?

    private var currentSnapshot = DataSourceSnapshot<String, String>()

    override init(frame: CGRect) {
        super.init(frame: frame)

        let spacer = UIView()
        spacer.translatesAutoresizingMaskIntoConstraints = false
        spacer.setContentHuggingPriority(.defaultLow - 1, for: .vertical)
        spacer.setContentCompressionResistancePriority(.defaultLow, for: .vertical)

        addViews()
        constraintViews()
        updateView(canContinue: false)
    }

    private func addViews() {
        addSubviews(
            scrollView,
            buttonStackView
        )

        scrollView.addSubview(contentView)

        contentView.addSubviews(
            statusImageView, titleLabel, messageLabel, deviceStackView
        )
    }

    private func constraintViews() {
        NSLayoutConstraint.activate([
            scrollView.topAnchor.constraint(equalTo: topAnchor),
            scrollView.leadingAnchor.constraint(equalTo: leadingAnchor),
            scrollView.trailingAnchor.constraint(equalTo: trailingAnchor),

            buttonStackView.topAnchor.constraint(equalTo: scrollView.bottomAnchor, constant: 24),
            buttonStackView.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 24),
            buttonStackView.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -24),
            buttonStackView.bottomAnchor.constraint(
                equalTo: safeAreaLayoutGuide.bottomAnchor,
                constant: -24
            ),
        ])

        NSLayoutConstraint.activate([
            contentView.topAnchor.constraint(equalTo: scrollView.topAnchor),
            contentView.leadingAnchor.constraint(equalTo: scrollView.leadingAnchor),
            contentView.trailingAnchor.constraint(equalTo: scrollView.trailingAnchor),
            contentView.bottomAnchor.constraint(equalTo: scrollView.bottomAnchor),
            contentView.widthAnchor.constraint(equalTo: scrollView.widthAnchor),
            contentView.heightAnchor.constraint(equalTo: scrollView.heightAnchor)
                .withPriority(.fittingSizeLevel),
        ])

        NSLayoutConstraint.activate([
            statusImageView.topAnchor.constraint(equalTo: contentView.layoutMarginsGuide.topAnchor),
            statusImageView.centerXAnchor.constraint(equalTo: contentView.centerXAnchor),

            titleLabel.topAnchor.constraint(equalTo: statusImageView.bottomAnchor, constant: 22),
            titleLabel.leadingAnchor
                .constraint(equalTo: contentView.layoutMarginsGuide.leadingAnchor),
            titleLabel.trailingAnchor
                .constraint(equalTo: contentView.layoutMarginsGuide.trailingAnchor),

            messageLabel.topAnchor.constraint(equalTo: titleLabel.bottomAnchor, constant: 8),
            messageLabel.leadingAnchor
                .constraint(equalTo: contentView.layoutMarginsGuide.leadingAnchor),
            messageLabel.trailingAnchor
                .constraint(equalTo: contentView.layoutMarginsGuide.trailingAnchor),

            deviceStackView.topAnchor.constraint(
                equalTo: messageLabel.bottomAnchor,
                constant: UIMetrics.sectionSpacing
            ),
            deviceStackView.leadingAnchor.constraint(equalTo: contentView.leadingAnchor),
            deviceStackView.trailingAnchor.constraint(equalTo: contentView.trailingAnchor),
            deviceStackView.bottomAnchor.constraint(equalTo: contentView.bottomAnchor),
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func setDeviceViewModels(_ newModels: [DeviceViewModel], canContinue: Bool) {
        updateView(canContinue: canContinue)

        var newSnapshot = DataSourceSnapshot<String, String>()
        newSnapshot.appendSections([""])
        newSnapshot.appendItems(newModels.map { $0.id }, in: "")

        let diff = currentSnapshot.difference(newSnapshot)
        currentSnapshot = newSnapshot

        let applyConfiguration = StackViewApplyDataSnapshotConfiguration { indexPath in
            let viewModel = newModels[indexPath.row]
            let view = DeviceRowView(viewModel: viewModel)
            view.deleteHandler = { [weak self] view in
                view.showsActivityIndicator = true

                self?.handleDeviceDeletion?(view.viewModel) {
                    view.showsActivityIndicator = false
                }
            }

            return view
        }

        diff.apply(
            to: deviceStackView,
            configuration: applyConfiguration,
            animateDifferences: true
        )
    }

    private func updateView(canContinue: Bool) {
        titleLabel.text = getTitleText(canContinue: canContinue)
        messageLabel.text = getMessageText(canContinue: canContinue)
        continueButton.isEnabled = canContinue
        statusImageView.style = canContinue ? .success : .failure
    }

    private func getTitleText(canContinue: Bool) -> String {
        if canContinue {
            return NSLocalizedString(
                "CONTINUE_LOGIN_TITLE",
                tableName: "DeviceManagement",
                value: "Super!",
                comment: ""
            )
        } else {
            return NSLocalizedString(
                "LOGOUT_DEVICES_TITLE",
                tableName: "DeviceManagement",
                value: "Too many devices",
                comment: ""
            )
        }
    }

    private func getMessageText(canContinue: Bool) -> String {
        if canContinue {
            return NSLocalizedString(
                "CONTINUE_LOGIN_MESSAGE",
                tableName: "DeviceManagement",
                value: "You can now continue logging in on this device.",
                comment: ""
            )
        } else {
            return NSLocalizedString(
                "LOGOUT_DEVICES_MESSAGE",
                tableName: "DeviceManagement",
                value: """
                Please log out of at least one by removing it from the list below. You can find \
                the corresponding device name under the device’s Account settings.
                """,
                comment: ""
            )
        }
    }
}

private extension UIView {
    func addSubviews(_ views: UIView...) {
        views.forEach(addSubview)
    }
}
