//
//  DeviceRowView.swift
//  MullvadVPN
//
//  Created by pronebird on 26/07/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import UIKit

class DeviceRowView: UIView {
    let viewModel: DeviceViewModel
    var deleteHandler: ((DeviceRowView) -> Void)?

    let textLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.font = UIFont.systemFont(ofSize: 17)
        textLabel.textColor = .white
        return textLabel
    }()

    let activityIndicator: SpinnerActivityIndicatorView = {
        let activityIndicator = SpinnerActivityIndicatorView(style: .custom)
        activityIndicator.translatesAutoresizingMaskIntoConstraints = false
        return activityIndicator
    }()

    let creationDateLabel: UILabel = {
        let creationDateLabel = UILabel()
        creationDateLabel.translatesAutoresizingMaskIntoConstraints = false
        creationDateLabel.font = UIFont.systemFont(ofSize: 14)
        creationDateLabel.textColor = .white.withAlphaComponent(0.6)
        return creationDateLabel
    }()

    let removeButton: UIButton = {
        let image = UIImage(named: "IconClose")?
            .withTintColor(
                .white.withAlphaComponent(0.4),
                renderingMode: .alwaysOriginal
            )

        let button = UIButton(type: .custom)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.setImage(image, for: .normal)
        button.accessibilityLabel = NSLocalizedString(
            "REMOVE_DEVICE_ACCESSIBILITY_LABEL",
            tableName: "DeviceManagement",
            value: "Remove device",
            comment: ""
        )
        return button
    }()

    var showsActivityIndicator = false {
        didSet {
            removeButton.isHidden = showsActivityIndicator

            if showsActivityIndicator {
                activityIndicator.startAnimating()
            } else {
                activityIndicator.stopAnimating()
            }
        }
    }

    init(viewModel: DeviceViewModel) {
        self.viewModel = viewModel

        super.init(frame: .zero)

        backgroundColor = .primaryColor
        directionalLayoutMargins = UIMetrics.TableView.rowViewLayoutMargins

        for subview in [textLabel, removeButton, activityIndicator, creationDateLabel] {
            addSubview(subview)
        }

        textLabel.text = viewModel.name
        creationDateLabel.text = .init(
            format:
            NSLocalizedString(
                "CREATED_DEVICE_LABEL",
                tableName: "DeviceManagement",
                value: "Created: %@",
                comment: ""
            ),
            viewModel.creationDate
        )

        removeButton.addTarget(self, action: #selector(handleButtonTap(_:)), for: .touchUpInside)

        NSLayoutConstraint.activate([
            textLabel.topAnchor.constraint(equalTo: layoutMarginsGuide.topAnchor),
            textLabel.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor),

            creationDateLabel.leadingAnchor.constraint(equalTo: textLabel.leadingAnchor),
            creationDateLabel.topAnchor.constraint(equalTo: textLabel.bottomAnchor, constant: 4.0),
            creationDateLabel.bottomAnchor.constraint(equalTo: layoutMarginsGuide.bottomAnchor)
                .withPriority(.defaultLow),
            creationDateLabel.trailingAnchor.constraint(equalTo: textLabel.trailingAnchor),

            removeButton.centerYAnchor.constraint(equalTo: layoutMarginsGuide.centerYAnchor),
            removeButton.leadingAnchor.constraint(
                greaterThanOrEqualTo: textLabel.trailingAnchor,
                constant: 8
            ),
            removeButton.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor),

            activityIndicator.centerXAnchor.constraint(equalTo: removeButton.centerXAnchor),
            activityIndicator.centerYAnchor.constraint(equalTo: removeButton.centerYAnchor),

            // Bump dimensions by 6pt to account for transparent pixels around spinner image.
            activityIndicator.widthAnchor.constraint(
                equalTo: removeButton.widthAnchor,
                constant: 6
            ),
            activityIndicator.heightAnchor.constraint(
                equalTo: removeButton.heightAnchor,
                constant: 6
            ),
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    @objc private func handleButtonTap(_ sender: Any?) {
        deleteHandler?(self)
    }
}
