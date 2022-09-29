//
//  DeviceManagementContentView.swift
//  MullvadVPN
//
//  Created by pronebird on 19/07/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import UIKit

class DeviceManagementContentView: UIView {
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

    var canContinue = false {
        didSet {
            updateView()
        }
    }

    var handleDeviceDeletion: ((DeviceViewModel, @escaping () -> Void) -> Void)?

    private var currentSnapshot = DataSourceSnapshot<String, String>()

    func setDeviceViewModels(_ newModels: [DeviceViewModel], animated: Bool) {
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

    override init(frame: CGRect) {
        super.init(frame: frame)

        layoutMargins = UIMetrics.contentLayoutMargins

        let spacer = UIView()
        spacer.translatesAutoresizingMaskIntoConstraints = false
        spacer.setContentHuggingPriority(.defaultLow - 1, for: .vertical)
        spacer.setContentCompressionResistancePriority(.defaultLow, for: .vertical)

        let subviewsToAdd = [
            statusImageView, titleLabel, messageLabel, deviceStackView,
        ]
        for subview in subviewsToAdd {
            addSubview(subview)
        }

        updateView()

        NSLayoutConstraint.activate([
            statusImageView.topAnchor.constraint(equalTo: layoutMarginsGuide.topAnchor),
            statusImageView.centerXAnchor.constraint(equalTo: centerXAnchor),

            titleLabel.topAnchor.constraint(equalTo: statusImageView.bottomAnchor, constant: 22),
            titleLabel.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor),
            titleLabel.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor),

            messageLabel.topAnchor.constraint(equalTo: titleLabel.bottomAnchor, constant: 8),
            messageLabel.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor),
            messageLabel.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor),

            deviceStackView.topAnchor.constraint(
                equalTo: messageLabel.bottomAnchor,
                constant: UIMetrics.sectionSpacing
            ),
            deviceStackView.leadingAnchor.constraint(equalTo: leadingAnchor),
            deviceStackView.trailingAnchor.constraint(equalTo: trailingAnchor),
            deviceStackView.bottomAnchor.constraint(equalTo: layoutMarginsGuide.bottomAnchor),
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func updateView() {
        titleLabel.text = titleText
        messageLabel.text = messageText
        statusImageView.style = canContinue ? .success : .failure
    }

    private var titleText: String {
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

    private var messageText: String {
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
