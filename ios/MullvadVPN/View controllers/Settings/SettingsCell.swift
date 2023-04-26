//
//  SettingsCell.swift
//  MullvadVPN
//
//  Created by pronebird on 22/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
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
            return nil
        case .chevron:
            return UIImage(named: "IconChevron")
        case .externalLink:
            return UIImage(named: "IconExtlink")
        case .tick:
            return UIImage(named: "IconTickSml")
        }
    }
}

class SettingsCell: UITableViewCell {
    typealias InfoButtonHandler = () -> Void

    let titleLabel = UILabel()
    let detailTitleLabel = UILabel()
    let disclosureImageView = UIImageView(image: nil)
    var infoButtonHandler: InfoButtonHandler?

    var disclosureType: SettingsDisclosureType = .none {
        didSet {
            accessoryType = .none

            let image = disclosureType.image?.withTintColor(
                UIColor.Cell.disclosureIndicatorColor,
                renderingMode: .alwaysOriginal
            )

            if let image = image {
                disclosureImageView.image = image
                disclosureImageView.sizeToFit()
                accessoryView = disclosureImageView
            } else {
                accessoryView = nil
            }
        }
    }

    private let buttonWidth: CGFloat = 24
    private let infoButton: UIButton = {
        let button = UIButton(type: .custom)
        button.accessibilityIdentifier = "InfoButton"
        button.tintColor = .white
        button.setImage(UIImage(named: "IconInfo"), for: .normal)
        return button
    }()

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: style, reuseIdentifier: reuseIdentifier)

        backgroundView = UIView()
        backgroundView?.backgroundColor = UIColor.Cell.backgroundColor

        selectedBackgroundView = UIView()
        selectedBackgroundView?.backgroundColor = UIColor.Cell.selectedAltBackgroundColor

        separatorInset = .zero
        backgroundColor = .clear
        contentView.backgroundColor = .clear

        infoButton.isHidden = true
        infoButton.addTarget(
            self,
            action: #selector(handleInfoButton(_:)),
            for: .touchUpInside
        )

        titleLabel.translatesAutoresizingMaskIntoConstraints = false
        titleLabel.font = UIFont.systemFont(ofSize: 17)
        titleLabel.textColor = UIColor.Cell.titleTextColor

        detailTitleLabel.translatesAutoresizingMaskIntoConstraints = false
        detailTitleLabel.font = UIFont.systemFont(ofSize: 13)
        detailTitleLabel.textColor = UIColor.Cell.detailTextColor

        titleLabel.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        detailTitleLabel.setContentHuggingPriority(.defaultLow, for: .horizontal)

        titleLabel.setContentCompressionResistancePriority(.defaultHigh, for: .horizontal)
        detailTitleLabel.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)

        setLayoutMargins()

        let buttonAreaWidth = UIMetrics.contentLayoutMargins.leading + UIMetrics
            .contentLayoutMargins.trailing + buttonWidth

        contentView.addConstrainedSubviews([titleLabel, infoButton, detailTitleLabel]) {
            switch style {
            case .subtitle:
                titleLabel.pinEdgesToSuperviewMargins(.init([.top(0), .leading(0)]))
                detailTitleLabel.pinEdgesToSuperviewMargins(.all().excluding(.top))
                detailTitleLabel.topAnchor.constraint(equalToSystemSpacingBelow: titleLabel.bottomAnchor, multiplier: 1)
                infoButton.trailingAnchor.constraint(greaterThanOrEqualTo: contentView.trailingAnchor)

            default:
                titleLabel.pinEdgesToSuperviewMargins(.all().excluding(.trailing))
                detailTitleLabel.pinEdgesToSuperviewMargins(.all().excluding(.leading))
                detailTitleLabel.leadingAnchor.constraint(greaterThanOrEqualTo: infoButton.trailingAnchor)
            }

            infoButton.pinEdgesToSuperview(.init([.top(0)]))
            infoButton.bottomAnchor.constraint(lessThanOrEqualTo: contentView.bottomAnchor)
            infoButton.leadingAnchor.constraint(
                equalTo: titleLabel.trailingAnchor,
                constant: -UIMetrics.interButtonSpacing
            )
            infoButton.centerYAnchor.constraint(equalTo: titleLabel.centerYAnchor)
            infoButton.widthAnchor.constraint(equalToConstant: buttonAreaWidth)
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func prepareForReuse() {
        super.prepareForReuse()

        setInfoButtonIsVisible(false)
        setLayoutMargins()
    }

    func applySubCellStyling() {
        contentView.layoutMargins.left += UIMetrics.cellIndentationWidth
        backgroundView?.backgroundColor = UIColor.SubCell.backgroundColor
    }

    func setInfoButtonIsVisible(_ visible: Bool) {
        infoButton.isHidden = !visible
    }

    @objc private func handleInfoButton(_ sender: UIControl) {
        infoButtonHandler?()
    }

    private func setLayoutMargins() {
        // Set layout margins for standard acceessories added into the cell (reorder control, etc..)
        directionalLayoutMargins = UIMetrics.settingsCellLayoutMargins

        // Set layout margins for cell content
        contentView.directionalLayoutMargins = UIMetrics.settingsCellLayoutMargins
    }
}
