//
//  SettingsCell.swift
//  MullvadVPN
//
//  Created by pronebird on 22/05/2019.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

enum SettingsDisclosureType {
    case none
    case chevron
    case externalLink
    case tick

    var image: UIImage? {
        switch self {
        case .none:
            nil
        case .chevron:
            UIImage.CellDecoration.chevronRight
        case .externalLink:
            UIImage.CellDecoration.externalLink
        case .tick:
            UIImage.CellDecoration.tick
        }
    }
}

class SettingsCell: UITableViewCell, CustomCellDisclosureHandling {
    typealias InfoButtonHandler = () -> Void

    let disclosureImageView = UIImageView(image: nil)
    let mainContentContainer = UIView()
    let leftContentContainer = UIView()
    let rightContentContainer = UIView()
    var infoButtonHandler: InfoButtonHandler? { didSet {
        infoButton.isHidden = infoButtonHandler == nil
    }}

    var disclosureType: SettingsDisclosureType = .none {
        didSet {
            accessoryType = .none

            let image = disclosureType.image?.withTintColor(
                UIColor.Cell.disclosureIndicatorColor,
                renderingMode: .alwaysOriginal
            )

            if let image {
                disclosureImageView.image = image
                disclosureImageView.sizeToFit()
                accessoryView = disclosureImageView
            } else {
                accessoryView = nil
            }
        }
    }

    let titleLabel: UILabel = {
        let label = UILabel()
        label.font = UIFont.systemFont(ofSize: 17)
        label.textColor = UIColor.Cell.titleTextColor
        label.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        label.setContentCompressionResistancePriority(.defaultHigh, for: .horizontal)
        return label
    }()

    let detailTitleLabel: UILabel = {
        let label = UILabel()
        label.font = UIFont.systemFont(ofSize: 13)
        label.textColor = UIColor.Cell.detailTextColor
        label.setContentHuggingPriority(.defaultLow, for: .horizontal)
        label.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        return label
    }()

    private var subCellLeadingIndentation: CGFloat = 0
    private let buttonWidth: CGFloat = 24

    private let infoButton: UIButton = {
        let button = UIButton(type: .custom)
        button.setAccessibilityIdentifier(.infoButton)
        button.tintColor = .white
        button.setImage(UIImage.Buttons.info, for: .normal)
        button.isHidden = true
        return button
    }()

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: style, reuseIdentifier: reuseIdentifier)

        backgroundView = UIView()
        backgroundView?.backgroundColor = UIColor.Cell.Background.normal

        selectedBackgroundView = UIView()
        selectedBackgroundView?.backgroundColor = UIColor.Cell.Background.selectedAlt

        separatorInset = .zero
        backgroundColor = .clear
        contentView.backgroundColor = .clear

        infoButton.addTarget(self, action: #selector(handleInfoButton(_:)), for: .touchUpInside)

        subCellLeadingIndentation = contentView.layoutMargins.left + UIMetrics.TableView.cellIndentationWidth

        rightContentContainer.setContentHuggingPriority(.required, for: .horizontal)

        setLayoutMargins()

        let buttonAreaWidth = UIMetrics.contentLayoutMargins.leading + UIMetrics
            .contentLayoutMargins.trailing + buttonWidth

        let infoButtonConstraint = infoButton.trailingAnchor.constraint(
            greaterThanOrEqualTo: mainContentContainer.trailingAnchor
        )
        infoButtonConstraint.priority = .defaultLow

        mainContentContainer.addConstrainedSubviews([titleLabel, infoButton, detailTitleLabel]) {
            switch style {
            case .subtitle:
                titleLabel.pinEdgesToSuperview(.init([.top(0), .leading(0)]))
                detailTitleLabel.pinEdgesToSuperview(.all().excluding([.top, .trailing]))
                detailTitleLabel.topAnchor.constraint(equalTo: titleLabel.bottomAnchor)
                infoButtonConstraint

            default:
                titleLabel.pinEdgesToSuperview(.all().excluding(.trailing))
                detailTitleLabel.pinEdgesToSuperview(.all().excluding(.leading))
                detailTitleLabel.leadingAnchor.constraint(greaterThanOrEqualTo: infoButton.trailingAnchor)
            }

            infoButton.leadingAnchor.constraint(
                equalTo: titleLabel.trailingAnchor,
                constant: -UIMetrics.interButtonSpacing
            )
            infoButton.centerYAnchor.constraint(equalTo: titleLabel.centerYAnchor)
            infoButton.widthAnchor.constraint(equalToConstant: buttonAreaWidth)
        }

        contentView.addConstrainedSubviews([leftContentContainer, mainContentContainer, rightContentContainer]) {
            mainContentContainer.pinEdgesToSuperviewMargins(.all().excluding([.leading, .trailing]))

            leftContentContainer.pinEdgesToSuperviewMargins(.all().excluding(.trailing))
            leftContentContainer.trailingAnchor.constraint(equalTo: mainContentContainer.leadingAnchor)

            rightContentContainer.pinEdgesToSuperview(.all().excluding(.leading))
            rightContentContainer.leadingAnchor.constraint(equalTo: mainContentContainer.trailingAnchor)
            rightContentContainer.widthAnchor.constraint(
                greaterThanOrEqualToConstant: UIMetrics.TableView.cellIndentationWidth
            )
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func prepareForReuse() {
        super.prepareForReuse()

        infoButton.isHidden = true
        removeLeadingView()
        removeTrailingView()
        setLayoutMargins()
    }

    func applySubCellStyling() {
        contentView.layoutMargins.left = subCellLeadingIndentation
        backgroundView?.backgroundColor = UIColor.Cell.Background.indentationLevelOne
    }

    func setLeadingView(superviewProvider: (UIView) -> Void) {
        removeLeadingView()
        superviewProvider(leftContentContainer)
    }

    func removeLeadingView() {
        leftContentContainer.subviews.forEach { $0.removeFromSuperview() }
    }

    func setTrailingView(superviewProvider: (UIView) -> Void) {
        removeTrailingView()
        superviewProvider(rightContentContainer)
    }

    func removeTrailingView() {
        rightContentContainer.subviews.forEach { $0.removeFromSuperview() }
    }

    @objc private func handleInfoButton(_ sender: UIControl) {
        infoButtonHandler?()
    }

    private func setLayoutMargins() {
        // Set layout margins for standard acceessories added into the cell (reorder control, etc..)
        directionalLayoutMargins = UIMetrics.SettingsCell.layoutMargins

        // Set layout margins for cell content
        contentView.directionalLayoutMargins = UIMetrics.SettingsCell.layoutMargins
    }
}
