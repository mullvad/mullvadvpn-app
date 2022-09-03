//
//  SelectLocationCell.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

private let kCollapseButtonWidth: CGFloat = 24
private let kRelayIndicatorSize: CGFloat = 16

class SelectLocationCell: UITableViewCell {
    typealias CollapseHandler = (SelectLocationCell) -> Void

    private let locationLabel = UILabel()
    private let statusIndicator: UIView = {
        let view = UIView()
        view.layer.cornerRadius = kRelayIndicatorSize * 0.5
        if #available(iOS 13.0, *) {
            view.layer.cornerCurve = .circular
        }
        return view
    }()

    private let tickImageView = UIImageView(image: UIImage(named: "IconTick"))
    private let collapseButton = UIButton(type: .custom)
    private let pinImageView = UIImageView(image: UIImage(named: "IconPinned"))

    private let chevronDown = UIImage(named: "IconChevronDown")
    private let chevronUp = UIImage(named: "IconChevronUp")

    func setLocationText(_ text: String, highlightedText: String) {
        let highlightColor = UIColor.Cell.titleTextColor
        let foregroundColor: UIColor = highlightedText.isEmpty
            ? highlightColor
            : highlightColor.withAlphaComponent(0.6)

        let string = NSMutableAttributedString(
            string: text,
            attributes: [
                .foregroundColor: foregroundColor,
                .font: UIFont.systemFont(ofSize: 17),
            ]
        )

        let highlightedRange = NSString(string: text).range(
            of: highlightedText,
            options: [.anchored, .caseInsensitive, .diacriticInsensitive]
        )
        string.addAttributes(
            [
                .foregroundColor: highlightColor,
                .font: UIFont.systemFont(ofSize: 17, weight: .semibold),
            ],
            range: highlightedRange
        )

        locationLabel.attributedText = string
    }

    var isDisabled = false {
        didSet {
            updateDisabled()
            updateBackgroundColor()
            updateStatusIndicatorColor()
        }
    }

    var isPinned = false {
        didSet {
            pinImageView.isHidden = !isPinned
        }
    }

    var isTableViewSwiped = false

    var isExpanded = false {
        didSet {
            updateCollapseImage()
            updateAccessibilityCustomActions()
        }
    }

    var showsCollapseControl = false {
        didSet {
            collapseButton.isHidden = !showsCollapseControl
            updateAccessibilityCustomActions()
        }
    }

    var didCollapseHandler: CollapseHandler?

    override var indentationLevel: Int {
        didSet {
            updateBackgroundColor()
            setLayoutMargins()
        }
    }

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: style, reuseIdentifier: reuseIdentifier)

        setupCell()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func setLayoutMargins() {
        let indentation = CGFloat(indentationLevel) * indentationWidth

        var contentMargins = UIMetrics.selectLocationCellLayoutMargins
        contentMargins.left += indentation

        contentView.layoutMargins = contentMargins
    }

    override func setHighlighted(_ highlighted: Bool, animated: Bool) {
        super.setHighlighted(highlighted, animated: animated)

        updateStatusIndicatorColor()
    }

    override func setSelected(_ selected: Bool, animated: Bool) {
        guard !isTableViewSwiped else { return }
        super.setSelected(selected, animated: animated)

        updateTickImage()
        updateStatusIndicatorColor()
    }

    private func setupCell() {
        indentationWidth = UIMetrics.cellIndentationWidth

        backgroundColor = .clear
        contentView.backgroundColor = .clear

        backgroundView = UIView()
        backgroundView?.backgroundColor = UIColor.Cell.backgroundColor

        selectedBackgroundView = UIView()
        selectedBackgroundView?.backgroundColor = UIColor.Cell.selectedBackgroundColor

        locationLabel.lineBreakMode = .byWordWrapping
        locationLabel.numberOfLines = 0
        if #available(iOS 14.0, *) {
            // See: https://stackoverflow.com/q/46200027/351305
            locationLabel.lineBreakStrategy = []
        }

        tickImageView.tintColor = .white

        pinImageView.tintColor = .white
        pinImageView.contentMode = .scaleAspectFit

        collapseButton.accessibilityIdentifier = "CollapseButton"
        collapseButton.isAccessibilityElement = false
        collapseButton.tintColor = .white
        collapseButton.addTarget(
            self,
            action: #selector(handleCollapseButton(_:)),
            for: .touchUpInside
        )

        [locationLabel, tickImageView, statusIndicator, pinImageView, collapseButton]
            .forEach { subview in
                subview.translatesAutoresizingMaskIntoConstraints = false
                contentView.addSubview(subview)
            }

        updateCollapseImage()
        updateAccessibilityCustomActions()
        updateDisabled()
        updateBackgroundColor()
        setLayoutMargins()

        NSLayoutConstraint.activate([
            pinImageView.leadingAnchor.constraint(equalTo: contentView.leadingAnchor, constant: 8),
            pinImageView.widthAnchor.constraint(equalToConstant: kRelayIndicatorSize),
            pinImageView.topAnchor.constraint(equalTo: contentView.topAnchor),
            pinImageView.bottomAnchor.constraint(equalTo: contentView.bottomAnchor),

            tickImageView.leadingAnchor
                .constraint(equalTo: contentView.layoutMarginsGuide.leadingAnchor),
            tickImageView.centerYAnchor.constraint(equalTo: contentView.centerYAnchor),

            statusIndicator.widthAnchor.constraint(equalToConstant: kRelayIndicatorSize),
            statusIndicator.heightAnchor.constraint(equalTo: statusIndicator.widthAnchor),
            statusIndicator.centerXAnchor.constraint(equalTo: tickImageView.centerXAnchor),
            statusIndicator.centerYAnchor.constraint(equalTo: tickImageView.centerYAnchor),

            locationLabel.leadingAnchor.constraint(
                equalTo: statusIndicator.trailingAnchor,
                constant: 12
            ),
            locationLabel.trailingAnchor.constraint(
                lessThanOrEqualTo: collapseButton.leadingAnchor
            ),
            locationLabel.topAnchor.constraint(equalTo: contentView.layoutMarginsGuide.topAnchor),
            locationLabel.bottomAnchor.constraint(
                equalTo: contentView.layoutMarginsGuide.bottomAnchor
            ),

            collapseButton.widthAnchor
                .constraint(
                    equalToConstant: UIMetrics.contentLayoutMargins.left + UIMetrics
                        .contentLayoutMargins.right + kCollapseButtonWidth
                ),
            collapseButton.topAnchor.constraint(equalTo: contentView.topAnchor),
            collapseButton.trailingAnchor.constraint(equalTo: contentView.trailingAnchor),
            collapseButton.bottomAnchor.constraint(equalTo: contentView.bottomAnchor),
        ])
    }

    private func updateTickImage() {
        statusIndicator.isHidden = isSelected
        tickImageView.isHidden = !isSelected
    }

    private func updateStatusIndicatorColor() {
        statusIndicator.backgroundColor = statusIndicatorColor()
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
        backgroundView?.backgroundColor = backgroundColorForIndentationLevel()
        selectedBackgroundView?.backgroundColor = selectedBackgroundColorForIndentationLevel()
    }

    private func backgroundColorForIndentationLevel() -> UIColor {
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

    private func statusIndicatorColor() -> UIColor {
        if isDisabled {
            return UIColor.RelayStatusIndicator.inactiveColor
        } else if isHighlighted {
            return UIColor.RelayStatusIndicator.highlightColor
        } else {
            return UIColor.RelayStatusIndicator.activeColor
        }
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
        if showsCollapseControl {
            let actionName = isExpanded
                ? NSLocalizedString(
                    "SELECT_LOCATION_COLLAPSE_ACCESSIBILITY_ACTION",
                    tableName: "SelectLocation",
                    value: "Collapse location",
                    comment: ""
                )
                : NSLocalizedString(
                    "SELECT_LOCATION_EXPAND_ACCESSIBILITY_ACTION",
                    tableName: "SelectLocation",
                    value: "Expand location",
                    comment: ""
                )

            accessibilityCustomActions = [
                UIAccessibilityCustomAction(
                    name: actionName,
                    target: self,
                    selector: #selector(toggleCollapseAccessibilityAction)
                ),
            ]
        } else {
            accessibilityCustomActions = nil
        }
    }
}
