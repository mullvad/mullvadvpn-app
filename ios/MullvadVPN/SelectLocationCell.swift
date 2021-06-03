//
//  SelectLocationCell.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

private let kCollapseButtonWidth: CGFloat = 24

class SelectLocationCell: BasicTableViewCell {
    typealias CollapseHandler = (SelectLocationCell) -> Void

    let locationLabel = UILabel()
    let statusIndicator = RelayStatusIndicatorView()
    let tickImageView = UIImageView(image: UIImage(named: "IconTick"))
    let collapseButton = UIButton(type: .custom)

    private let chevronDown = UIImage(named: "IconChevronDown")
    private let chevronUp = UIImage(named: "IconChevronUp")

    var isDisabled = false {
        didSet {
            updateDisabled()
            updateBackgroundColor()
        }
    }

    var isExpanded = false {
        didSet {
            updateCollapseImage()
        }
    }

    var showsCollapseControl = false {
        didSet {
            collapseButton.isHidden = !showsCollapseControl
        }
    }

    var didCollapseHandler: CollapseHandler?

    private let preferredMargins = UIEdgeInsets(top: 16, left: 28, bottom: 16, right: 12)

    override var indentationLevel: Int {
        didSet {
            updateBackgroundColor()
        }
    }

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: style, reuseIdentifier: reuseIdentifier)

        setupCell()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func layoutSubviews() {
        super.layoutSubviews()

        let indentPoints = CGFloat(indentationLevel) * indentationWidth

        contentView.frame = CGRect(
            x: indentPoints,
            y: contentView.frame.origin.y,
            width: contentView.frame.size.width - indentPoints,
            height: contentView.frame.size.height
        )
    }

    override func setSelected(_ selected: Bool, animated: Bool) {
        super.setSelected(selected, animated: animated)

        updateTickImage()
    }

    private func setupCell() {
        indentationWidth = 16

        backgroundColor = .clear
        contentView.layoutMargins = preferredMargins

        locationLabel.font = UIFont.systemFont(ofSize: 17)
        locationLabel.textColor = .white

        statusIndicator.tintColor = .white
        tickImageView.tintColor = .white

        collapseButton.accessibilityIdentifier = "CollapseButton"
        collapseButton.tintColor = .white
        collapseButton.setImage(chevronDown, for: .normal)
        collapseButton.addTarget(self, action: #selector(handleCollapseButton(_ :)), for: .touchUpInside)

        [locationLabel, tickImageView, statusIndicator, collapseButton].forEach { (subview) in
            subview.translatesAutoresizingMaskIntoConstraints = false
            contentView.addSubview(subview)
        }

        updateCollapseImage()
        updateDisabled()
        updateBackgroundColor()

        NSLayoutConstraint.activate([
            tickImageView.leadingAnchor.constraint(equalTo: contentView.layoutMarginsGuide.leadingAnchor),
            tickImageView.centerYAnchor.constraint(equalTo: contentView.centerYAnchor),

            statusIndicator.widthAnchor.constraint(equalToConstant: 16),
            statusIndicator.heightAnchor.constraint(equalToConstant: 16),
            statusIndicator.centerXAnchor.constraint(equalTo: tickImageView.centerXAnchor),
            statusIndicator.centerYAnchor.constraint(equalTo: tickImageView.centerYAnchor),

            locationLabel.leadingAnchor.constraint(equalTo: statusIndicator.trailingAnchor, constant: 12),
            locationLabel.trailingAnchor.constraint(greaterThanOrEqualTo: collapseButton.leadingAnchor, constant: 0),
            locationLabel.topAnchor.constraint(equalTo: contentView.layoutMarginsGuide.topAnchor),
            locationLabel.bottomAnchor.constraint(equalTo: contentView.layoutMarginsGuide.bottomAnchor),

            collapseButton.widthAnchor.constraint(equalToConstant: UIMetrics.contentLayoutMargins.left + UIMetrics.contentLayoutMargins.right + kCollapseButtonWidth),
            collapseButton.topAnchor.constraint(equalTo: contentView.topAnchor),
            collapseButton.trailingAnchor.constraint(equalTo: contentView.trailingAnchor),
            collapseButton.bottomAnchor.constraint(equalTo: contentView.bottomAnchor)
        ])
    }

    private func updateTickImage() {
        statusIndicator.isHidden = isSelected
        tickImageView.isHidden = !isSelected
    }

    private func updateDisabled() {
        locationLabel.alpha = isDisabled ? 0.2 : 1
        collapseButton.alpha = isDisabled ? 0.2 : 1

        if isDisabled {
            accessibilityTraits.insert(.notEnabled)
        } else {
            accessibilityTraits.remove(.notEnabled)
        }
    }

    private func updateBackgroundColor() {
        backgroundView?.backgroundColor = backgroundColorForIdentationLevel()
        selectedBackgroundView?.backgroundColor = selectedBackgroundColorForIndentationLevel()
    }

    private func backgroundColorForIdentationLevel() -> UIColor {
        switch indentationLevel {
        case 1:
            return UIColor.SubCell.backgroundColor
        case 2:
            return UIColor.SubSubCell.backgroundColor
        default:
            return UIColor.Cell.backgroundColor
        }
    }

    private func selectedBackgroundColorForIndentationLevel() -> UIColor {
        if isDisabled {
            return UIColor.Cell.disabledSelectedBackgroundColor
        } else {
            return UIColor.Cell.selectedBackgroundColor
        }
    }

    @objc private func handleCollapseButton(_ sender: UIControl) {
        didCollapseHandler?(self)
    }

    private func updateCollapseImage() {
        let image = isExpanded ? chevronUp : chevronDown

        collapseButton.setImage(image, for: .normal)
    }
}
