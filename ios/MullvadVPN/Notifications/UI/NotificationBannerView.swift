//
//  NotificationBannerView.swift
//  MullvadVPN
//
//  Created by pronebird on 01/06/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

final class NotificationBannerView: UIView {
    private let backgroundView = UIVisualEffectView(effect: UIBlurEffect(style: .dark))

    private let titleLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.font = UIFont.systemFont(ofSize: 17, weight: .bold)
        textLabel.textColor = UIColor.InAppNotificationBanner.titleColor
        textLabel.numberOfLines = 0
        textLabel.lineBreakMode = .byWordWrapping
        textLabel.lineBreakStrategy = []
        return textLabel
    }()

    private let bodyLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.font = UIFont.systemFont(ofSize: 17)
        textLabel.textColor = UIColor.InAppNotificationBanner.bodyColor
        textLabel.numberOfLines = 0
        textLabel.lineBreakMode = .byWordWrapping
        textLabel.lineBreakStrategy = []
        return textLabel
    }()

    private let indicatorView: UIView = {
        let view = UIView()
        view.backgroundColor = .dangerColor
        view.layer.cornerRadius = UIMetrics.InAppBannerNotification.indicatorSize.width * 0.5
        view.layer.cornerCurve = .circular
        return view
    }()

    private let wrapperView: UIView = {
        let view = UIView()
        view.directionalLayoutMargins = UIMetrics.InAppBannerNotification.layoutMargins
        return view
    }()

    private lazy var bodyStackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [titleLabel, bodyLabel])
        stackView.alignment = .top
        stackView.distribution = .fill
        stackView.axis = .vertical
        stackView.spacing = UIStackView.spacingUseSystem
        return stackView
    }()

    private lazy var contentStackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [bodyStackView, actionButton])
        stackView.spacing = UIStackView.spacingUseSystem
        stackView.alignment = .center
        return stackView
    }()

    private let actionButton: IncreasedHitButton = {
        let button = IncreasedHitButton(type: .system)
        button.tintColor = UIColor.InAppNotificationBanner.actionButtonColor
        return button
    }()

    var title: String? {
        didSet {
            titleLabel.text = title
        }
    }

    var body: NSAttributedString? {
        didSet {
            bodyLabel.attributedText = body
        }
    }

    var style: NotificationBannerStyle = .error {
        didSet {
            indicatorView.backgroundColor = style.color
        }
    }

    var action: InAppNotificationAction? {
        didSet {
            let image = action?.image
            let showsAction = image != nil

            actionButton.setBackgroundImage(image, for: .normal)
            actionButton.widthAnchor.constraint(equalToConstant: 24).isActive = true
            actionButton.heightAnchor.constraint(equalTo: actionButton.widthAnchor).isActive = true
            actionButton.isHidden = !showsAction
        }
    }

    var tapAction: InAppNotificationAction?

    override init(frame: CGRect) {
        super.init(frame: frame)
        addSubviews()
        addTapHandler()
        addActionHandlers()
        addConstraints()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func addTapHandler() {
        let tapGesture = UITapGestureRecognizer(target: self, action: #selector(handleTap))
        addGestureRecognizer(tapGesture)
    }

    private func addActionHandlers() {
        actionButton.addTarget(self, action: #selector(handleActionTap), for: .touchUpInside)
    }

    @objc
    private func handleTap() {
        tapAction?.handler?()
    }

    private func addSubviews() {
        wrapperView.addConstrainedSubviews([indicatorView, contentStackView])
        backgroundView.contentView.addConstrainedSubviews([wrapperView]) {
            wrapperView.pinEdgesToSuperview()
        }
        addConstrainedSubviews([backgroundView]) {
            backgroundView.pinEdgesToSuperview()
        }
    }

    private func addConstraints() {
        NSLayoutConstraint.activate([
            indicatorView.bottomAnchor.constraint(equalTo: titleLabel.firstBaselineAnchor),
            indicatorView.leadingAnchor.constraint(equalTo: wrapperView.layoutMarginsGuide.leadingAnchor),
            indicatorView.widthAnchor
                .constraint(equalToConstant: UIMetrics.InAppBannerNotification.indicatorSize.width),
            indicatorView.heightAnchor
                .constraint(equalToConstant: UIMetrics.InAppBannerNotification.indicatorSize.height),

            contentStackView.topAnchor.constraint(equalTo: wrapperView.layoutMarginsGuide.topAnchor),
            contentStackView.leadingAnchor.constraint(
                equalToSystemSpacingAfter: indicatorView.trailingAnchor,
                multiplier: 1
            ),
            contentStackView.trailingAnchor.constraint(equalTo: wrapperView.layoutMarginsGuide.trailingAnchor),
            contentStackView.bottomAnchor.constraint(equalTo: wrapperView.layoutMarginsGuide.bottomAnchor),
        ])
    }

    @objc private func handleActionTap() {
        action?.handler?()
    }
}

private extension NotificationBannerStyle {
    var color: UIColor {
        switch self {
        case .success:
            return UIColor.InAppNotificationBanner.successIndicatorColor
        case .warning:
            return UIColor.InAppNotificationBanner.warningIndicatorColor
        case .error:
            return UIColor.InAppNotificationBanner.errorIndicatorColor
        }
    }
}
