//
//  SettingsContentBlockersHeaderView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-04-06.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

class SettingsContentBlockersHeaderView: UITableViewHeaderFooterView {
    typealias InfoButtonHandler = () -> Void
    typealias CollapseHandler = (SettingsContentBlockersHeaderView) -> Void

    let titleLabel: UILabel = {
        let titleLabel = UILabel()
        titleLabel.translatesAutoresizingMaskIntoConstraints = false
        titleLabel.font = .systemFont(ofSize: 17)
        titleLabel.textColor = UIColor.Cell.titleTextColor
        titleLabel.numberOfLines = 0
        return titleLabel
    }()

    let infoButton: UIButton = {
        let button = UIButton(type: .custom)
        button.accessibilityIdentifier = "InfoButton"
        button.tintColor = .white
        button.setImage(UIImage(named: "IconInfo"), for: .normal)
        return button
    }()

    let collapseButton: UIButton = {
        let button = UIButton(type: .custom)
        button.accessibilityIdentifier = "CollapseButton"
        button.isAccessibilityElement = false
        button.tintColor = .white
        return button
    }()

    var isExpanded = false {
        didSet {
            updateCollapseImage()
            updateAccessibilityCustomActions()
        }
    }

    var didCollapseHandler: CollapseHandler?
    var infoButtonHandler: InfoButtonHandler?

    private let chevronDown = UIImage(named: "IconChevronDown")
    private let chevronUp = UIImage(named: "IconChevronUp")
    private let buttonWidth: CGFloat = 24

    override init(reuseIdentifier: String?) {
        super.init(reuseIdentifier: reuseIdentifier)

        infoButton.addTarget(
            self,
            action: #selector(handleInfoButton(_:)),
            for: .touchUpInside
        )

        collapseButton.addTarget(
            self,
            action: #selector(handleCollapseButton(_:)),
            for: .touchUpInside
        )

        contentView.directionalLayoutMargins = UIMetrics.settingsCellLayoutMargins
        contentView.backgroundColor = UIColor.Cell.backgroundColor

        let buttonAreaWidth = UIMetrics.contentLayoutMargins.leading + UIMetrics
            .contentLayoutMargins.trailing + buttonWidth

        contentView.addConstrainedSubviews([titleLabel, infoButton, collapseButton]) {
            titleLabel.pinEdgesToSuperviewMargins(.all().excluding(.trailing).excluding(.bottom))
            titleLabel.bottomAnchor.constraint(
                equalTo: contentView.bottomAnchor,
                constant: -contentView.layoutMargins.bottom
            ).withPriority(.defaultHigh)

            infoButton.pinEdgesToSuperview(.init([.top(0), .bottom(0)]))
            infoButton.leadingAnchor.constraint(
                equalTo: titleLabel.trailingAnchor,
                constant: -UIMetrics.interButtonSpacing
            )
            infoButton.widthAnchor.constraint(equalToConstant: buttonAreaWidth)

            collapseButton.pinEdgesToSuperview(.all().excluding(.leading))
            collapseButton.leadingAnchor.constraint(greaterThanOrEqualTo: infoButton.trailingAnchor)
            collapseButton.widthAnchor.constraint(equalToConstant: buttonAreaWidth)
        }

        updateCollapseImage()
        updateAccessibilityCustomActions()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    @objc private func handleInfoButton(_ sender: UIControl) {
        infoButtonHandler?()
    }

    @objc private func handleCollapseButton(_ sender: UIControl) {
        didCollapseHandler?(self)
    }

    @objc private func toggleCollapseAccessibilityAction() -> Bool {
        didCollapseHandler?(self)
        return true
    }

    private func updateCollapseImage() {
        let image = isExpanded ? chevronUp : chevronDown

        collapseButton.setImage(image, for: .normal)
    }

    private func updateAccessibilityCustomActions() {
        let actionName = isExpanded
            ? NSLocalizedString(
                "CONTENT_BLOCKERS_COLLAPSE_ACCESSIBILITY_ACTION",
                tableName: "Settings",
                value: "Collapse content blockers",
                comment: ""
            )
            : NSLocalizedString(
                "CONTENT_BLOCKERS_EXPAND_ACCESSIBILITY_ACTION",
                tableName: "Settings",
                value: "Expand content blockers",
                comment: ""
            )

        accessibilityCustomActions = [
            UIAccessibilityCustomAction(
                name: actionName,
                target: self,
                selector: #selector(toggleCollapseAccessibilityAction)
            ),
        ]
    }
}
