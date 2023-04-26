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

    let scrollContentView: UIView = {
        let view = UIView()
        view.directionalLayoutMargins = UIMetrics.contentLayoutMargins
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

    private var currentDeviceModels = [DeviceViewModel]()

    var canContinue = false {
        didSet {
            updateView()
        }
    }

    override init(frame: CGRect) {
        super.init(frame: frame)

        addViews()
        constraintViews()
        updateView()
    }

    private func addViews() {
        [scrollView, buttonStackView].forEach(addSubview)

        scrollView.addSubview(scrollContentView)

        [statusImageView, titleLabel, messageLabel, deviceStackView]
            .forEach(scrollContentView.addSubview)
    }

    private func constraintViews() {
        NSLayoutConstraint.activate([
            scrollView.topAnchor.constraint(equalTo: topAnchor),
            scrollView.leadingAnchor.constraint(equalTo: leadingAnchor),
            scrollView.trailingAnchor.constraint(equalTo: trailingAnchor),

            buttonStackView.topAnchor.constraint(
                equalTo: scrollView.bottomAnchor,
                constant: UIMetrics.contentLayoutMargins.top
            ),
            buttonStackView.leadingAnchor.constraint(
                equalTo: leadingAnchor,
                constant: UIMetrics.contentLayoutMargins.leading
            ),
            buttonStackView.trailingAnchor.constraint(
                equalTo: trailingAnchor,
                constant: -UIMetrics.contentLayoutMargins.trailing
            ),
            buttonStackView.bottomAnchor.constraint(
                equalTo: safeAreaLayoutGuide.bottomAnchor,
                constant: -UIMetrics.contentLayoutMargins.bottom
            ),

            scrollContentView.topAnchor.constraint(equalTo: scrollView.topAnchor),
            scrollContentView.leadingAnchor.constraint(equalTo: scrollView.leadingAnchor),
            scrollContentView.trailingAnchor.constraint(equalTo: scrollView.trailingAnchor),
            scrollContentView.bottomAnchor.constraint(equalTo: scrollView.bottomAnchor),
            scrollContentView.widthAnchor.constraint(equalTo: scrollView.widthAnchor),

            statusImageView.topAnchor
                .constraint(equalTo: scrollContentView.layoutMarginsGuide.topAnchor),
            statusImageView.centerXAnchor.constraint(equalTo: scrollContentView.centerXAnchor),

            titleLabel.topAnchor.constraint(equalTo: statusImageView.bottomAnchor, constant: 22),
            titleLabel.leadingAnchor
                .constraint(equalTo: scrollContentView.layoutMarginsGuide.leadingAnchor),
            titleLabel.trailingAnchor
                .constraint(equalTo: scrollContentView.layoutMarginsGuide.trailingAnchor),

            messageLabel.topAnchor.constraint(equalTo: titleLabel.bottomAnchor, constant: 8),
            messageLabel.leadingAnchor
                .constraint(equalTo: scrollContentView.layoutMarginsGuide.leadingAnchor),
            messageLabel.trailingAnchor
                .constraint(equalTo: scrollContentView.layoutMarginsGuide.trailingAnchor),

            deviceStackView.topAnchor.constraint(
                equalTo: messageLabel.bottomAnchor,
                constant: UIMetrics.sectionSpacing
            ),
            deviceStackView.leadingAnchor.constraint(equalTo: scrollContentView.leadingAnchor),
            deviceStackView.trailingAnchor.constraint(equalTo: scrollContentView.trailingAnchor),
            deviceStackView.bottomAnchor.constraint(equalTo: scrollContentView.bottomAnchor),
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func setDeviceViewModels(_ newModels: [DeviceViewModel], animated: Bool) {
        let difference = newModels.difference(from: currentDeviceModels) { newModel, model in
            return newModel.id == model.id
        }

        currentDeviceModels = newModels

        var viewsToAdd: [(view: UIView, offset: Int)] = []
        var viewsToRemove: [UIView] = []

        difference.forEach { change in
            switch change {
            case let .insert(offset, model, _):
                let view = DeviceRowView(viewModel: model)

                view.isHidden = true
                view.alpha = 0

                view.deleteHandler = { [weak self] _ in
                    view.showsActivityIndicator = true

                    self?.handleDeviceDeletion?(view.viewModel) {
                        view.showsActivityIndicator = false
                    }
                }

                viewsToAdd.append((view, offset))

            case let .remove(offset, _, _):
                viewsToRemove.append(deviceStackView.arrangedSubviews[offset])
            }
        }

        viewsToAdd.forEach { item in
            deviceStackView.insertArrangedSubview(item.view, at: item.offset)
        }

        // Layout inserted subviews before running animations to achieve a folding effect.
        if animated {
            UIView.performWithoutAnimation {
                deviceStackView.layoutIfNeeded()
            }
        }

        let showHideViews = {
            viewsToRemove.forEach { view in
                view.alpha = 0
                view.isHidden = true
            }

            viewsToAdd.forEach { item in
                item.view.alpha = 1
                item.view.isHidden = false
            }
        }

        let removeViews = {
            viewsToRemove.forEach { view in
                view.removeFromSuperview()
            }
        }

        if animated {
            UIView.animate(
                withDuration: 0.25,
                delay: 0,
                options: [.curveEaseInOut],
                animations: { [weak self] in
                    showHideViews()
                    self?.deviceStackView.layoutIfNeeded()
                },
                completion: { isComplete in
                    removeViews()
                }
            )
        } else {
            showHideViews()
            removeViews()
        }
    }

    private func updateView() {
        titleLabel.text = titleText
        messageLabel.text = messageText
        continueButton.isEnabled = canContinue
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
