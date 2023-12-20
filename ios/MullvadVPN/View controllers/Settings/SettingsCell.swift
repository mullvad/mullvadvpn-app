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
            nil
        case .chevron:
            UIImage(resource: .iconChevron)
        case .externalLink:
            UIImage(resource: .iconExtlink)
        case .tick:
            UIImage(resource: .iconTickSml)
        }
    }
}

class SettingsCell: UITableViewCell, CustomCellDisclosureHandling {
    typealias InfoButtonHandler = () -> Void

    let contentContainerSubviewMaxCount = 2
    let titleLabel = UILabel()
    let detailTitleLabel = UILabel()
    let disclosureImageView = UIImageView(image: nil)
    let contentContainer = UIStackView()
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

    private let buttonWidth: CGFloat = 24
    private let infoButton: UIButton = {
        let button = UIButton(type: .custom)
        button.accessibilityIdentifier = .infoButton
        button.tintColor = .white
        button.setImage(UIImage(named: "IconInfo"), for: .normal)
        button.isHidden = true
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
        infoButton.addTarget(self, action: #selector(handleInfoButton(_:)), for: .touchUpInside)

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

        let content = UIView()
        content.addConstrainedSubviews([titleLabel, infoButton, detailTitleLabel]) {
            switch style {
            case .subtitle:
                titleLabel.pinEdgesToSuperview(.init([.top(0), .leading(0)]))
                detailTitleLabel.pinEdgesToSuperview(.all().excluding(.top))
                detailTitleLabel.topAnchor.constraint(equalToSystemSpacingBelow: titleLabel.bottomAnchor, multiplier: 1)
                infoButton.trailingAnchor.constraint(greaterThanOrEqualTo: content.trailingAnchor)

            default:
                titleLabel.pinEdgesToSuperview(.all().excluding(.trailing))
                detailTitleLabel.pinEdgesToSuperview(.all().excluding(.leading))
                detailTitleLabel.leadingAnchor.constraint(greaterThanOrEqualTo: infoButton.trailingAnchor)
            }

            infoButton.pinEdgesToSuperview(.init([.top(0)]))
            infoButton.bottomAnchor.constraint(lessThanOrEqualTo: content.bottomAnchor)
            infoButton.leadingAnchor.constraint(
                equalTo: titleLabel.trailingAnchor,
                constant: -UIMetrics.interButtonSpacing
            )
            infoButton.centerYAnchor.constraint(equalTo: titleLabel.centerYAnchor)
            infoButton.widthAnchor.constraint(equalToConstant: buttonAreaWidth)
        }

        contentContainer.addArrangedSubview(content)

        contentView.addConstrainedSubviews([contentContainer]) {
            contentContainer.pinEdgesToSuperviewMargins()
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func prepareForReuse() {
        super.prepareForReuse()

        infoButton.isHidden = true
        removeLeftView()
        setLayoutMargins()
    }

    func applySubCellStyling() {
        contentView.layoutMargins.left += UIMetrics.TableView.cellIndentationWidth
        backgroundView?.backgroundColor = UIColor.SubCell.backgroundColor
    }

    func setLeftView(_ view: UIView, spacing: CGFloat) {
        removeLeftView()

        if contentContainer.arrangedSubviews.count <= 1 {
            contentContainer.insertArrangedSubview(view, at: 0)
        }

        contentContainer.spacing = spacing
    }

    func removeLeftView() {
        if contentContainer.arrangedSubviews.count >= contentContainerSubviewMaxCount {
            contentContainer.arrangedSubviews.first?.removeFromSuperview()
        }
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
