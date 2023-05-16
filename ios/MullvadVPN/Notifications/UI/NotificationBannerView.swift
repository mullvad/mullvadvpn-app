//
//  NotificationBannerView.swift
//  MullvadVPN
//
//  Created by pronebird on 01/06/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

final class NotificationBannerView: UIView {
    private static let indicatorViewSize = CGSize(width: 12, height: 12)
    private static let buttonSize = CGSize(width: 18, height: 18)

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
        view.layer.cornerRadius = NotificationBannerView.indicatorViewSize.width * 0.5
        view.layer.cornerCurve = .circular
        return view
    }()

    private let wrapperView: UIView = {
        let view = UIView()
        view.directionalLayoutMargins = UIMetrics.inAppBannerNotificationLayoutMargins
        return view
    }()

    private let actionButton: UIButton = {
        let button = UIButton(type: .system)
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

            updateActionButtonConstraints(showsAction: showsAction)
        }
    }

    private var actionButtonVisibleConstraints: [NSLayoutConstraint] = []
    private var actionButtonHiddenConstraints: [NSLayoutConstraint] = []

    override init(frame: CGRect) {
        super.init(frame: frame)

        actionButton.addTarget(self, action: #selector(didPress), for: .touchUpInside)

        wrapperView.addConstrainedSubviews([titleLabel, bodyLabel, indicatorView, actionButton])
        backgroundView.contentView.addConstrainedSubviews([wrapperView]) {
            wrapperView.pinEdgesToSuperview()
        }
        addConstrainedSubviews([backgroundView]) {
            backgroundView.pinEdgesToSuperview()
        }

        actionButtonVisibleConstraints = [actionButton.leadingAnchor.constraint(equalTo: bodyLabel.trailingAnchor)]

        NSLayoutConstraint.activate([
            indicatorView.bottomAnchor.constraint(equalTo: titleLabel.firstBaselineAnchor),
            indicatorView.leadingAnchor.constraint(equalTo: wrapperView.layoutMarginsGuide.leadingAnchor),
            indicatorView.widthAnchor.constraint(equalToConstant: Self.indicatorViewSize.width),
            indicatorView.heightAnchor.constraint(equalToConstant: Self.indicatorViewSize.height),

            titleLabel.topAnchor.constraint(equalTo: wrapperView.layoutMarginsGuide.topAnchor),
            titleLabel.leadingAnchor.constraint(equalToSystemSpacingAfter: indicatorView.trailingAnchor, multiplier: 1),
            titleLabel.trailingAnchor.constraint(equalTo: wrapperView.layoutMarginsGuide.trailingAnchor),

            bodyLabel.topAnchor.constraint(equalToSystemSpacingBelow: titleLabel.bottomAnchor, multiplier: 1),
            bodyLabel.leadingAnchor.constraint(equalTo: titleLabel.leadingAnchor),
            bodyLabel.trailingAnchor.constraint(equalTo: wrapperView.layoutMarginsGuide.trailingAnchor)
                .store(into: &actionButtonHiddenConstraints),
            bodyLabel.bottomAnchor.constraint(equalTo: wrapperView.layoutMarginsGuide.bottomAnchor),

            actionButton.topAnchor.constraint(equalTo: bodyLabel.topAnchor),
            actionButton.trailingAnchor.constraint(equalTo: wrapperView.layoutMarginsGuide.trailingAnchor),
            actionButton.widthAnchor.constraint(equalToConstant: NotificationBannerView.buttonSize.width),
            actionButton.heightAnchor.constraint(equalToConstant: NotificationBannerView.buttonSize.height),
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    @objc private func didPress() {
        action?.handler?()
    }

    private func updateActionButtonConstraints(showsAction: Bool) {
        if showsAction {
            NSLayoutConstraint.deactivate(actionButtonHiddenConstraints)
            NSLayoutConstraint.activate(actionButtonVisibleConstraints)
        } else {
            NSLayoutConstraint.deactivate(actionButtonVisibleConstraints)
            NSLayoutConstraint.activate(actionButtonHiddenConstraints)
        }

        setNeedsLayout()
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
