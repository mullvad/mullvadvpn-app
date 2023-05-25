//
//  NotificationBannerView.swift
//  MullvadVPN
//
//  Created by pronebird on 01/06/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
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
        if #available(iOS 14.0, *) {
            // See: https://stackoverflow.com/q/46200027/351305
            textLabel.lineBreakStrategy = []
        }
        return textLabel
    }()

    private let bodyLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.font = UIFont.systemFont(ofSize: 17)
        textLabel.textColor = UIColor.InAppNotificationBanner.bodyColor
        textLabel.numberOfLines = 0
        textLabel.lineBreakMode = .byWordWrapping
        if #available(iOS 14.0, *) {
            // See: https://stackoverflow.com/q/46200027/351305
            textLabel.lineBreakStrategy = []
        }
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

    private let bodyStackView: UIStackView = {
        let stackView = UIStackView()
        stackView.alignment = .top
        stackView.distribution = .fill
        stackView.spacing = UIStackView.spacingUseSystem
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

            actionButton.setImage(image, for: .normal)
            actionButton.isHidden = !showsAction
        }
    }

    override init(frame: CGRect) {
        super.init(frame: frame)

        actionButton.addTarget(self, action: #selector(handleActionTap), for: .touchUpInside)

        actionButton.setContentCompressionResistancePriority(.defaultHigh + 1, for: .horizontal)
        actionButton.setContentCompressionResistancePriority(.defaultHigh + 1, for: .vertical)
        actionButton.setContentHuggingPriority(.defaultHigh + 1, for: .horizontal)
        actionButton.setContentHuggingPriority(.defaultHigh + 1, for: .vertical)

        wrapperView.addConstrainedSubviews([titleLabel, indicatorView, bodyStackView])
        backgroundView.contentView.addConstrainedSubviews([wrapperView]) {
            wrapperView.pinEdgesToSuperview()
        }
        addConstrainedSubviews([backgroundView]) {
            backgroundView.pinEdgesToSuperview()
        }

        bodyStackView.addArrangedSubview(bodyLabel)
        bodyStackView.addArrangedSubview(actionButton)

        NSLayoutConstraint.activate([
            indicatorView.bottomAnchor.constraint(equalTo: titleLabel.firstBaselineAnchor),
            indicatorView.leadingAnchor.constraint(equalTo: wrapperView.layoutMarginsGuide.leadingAnchor),
            indicatorView.widthAnchor
                .constraint(equalToConstant: UIMetrics.InAppBannerNotification.indicatorSize.width),
            indicatorView.heightAnchor
                .constraint(equalToConstant: UIMetrics.InAppBannerNotification.indicatorSize.height),

            titleLabel.topAnchor.constraint(equalTo: wrapperView.layoutMarginsGuide.topAnchor),
            titleLabel.leadingAnchor.constraint(equalToSystemSpacingAfter: indicatorView.trailingAnchor, multiplier: 1),
            titleLabel.trailingAnchor.constraint(equalTo: wrapperView.layoutMarginsGuide.trailingAnchor),

            bodyStackView.topAnchor.constraint(equalToSystemSpacingBelow: titleLabel.bottomAnchor, multiplier: 1),
            bodyStackView.leadingAnchor.constraint(equalTo: titleLabel.leadingAnchor),
            bodyStackView.trailingAnchor.constraint(equalTo: titleLabel.trailingAnchor),
            bodyStackView.bottomAnchor.constraint(equalTo: wrapperView.layoutMarginsGuide.bottomAnchor),
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
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
