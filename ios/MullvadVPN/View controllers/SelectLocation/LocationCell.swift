//
//  LocationCell.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

protocol LocationCellDelegate: AnyObject {
    func toggle(cell: LocationCell)
}

class LocationCell: UITableViewCell {
    weak var delegate: LocationCellDelegate?

    private let locationLabel: UILabel = {
        let label = UILabel()
        label.font = UIFont.systemFont(ofSize: 16)
        label.textColor = .white
        label.lineBreakMode = .byWordWrapping
        label.numberOfLines = 0
        label.lineBreakStrategy = []
        return label
    }()

    private let statusIndicator: UIView = {
        let view = UIView()
        view.layer.cornerRadius = 8
        view.layer.cornerCurve = .circular
        return view
    }()

    private let tickImageView: UIImageView = {
        let imageView = UIImageView(image: UIImage(resource: .iconTick))
        imageView.tintColor = .white
        return imageView
    }()

    private let collapseButton: UIButton = {
        let button = UIButton(type: .custom)
        button.isAccessibilityElement = false
        button.tintColor = .white
        return button
    }()

    private let chevronDown = UIImage(resource: .iconChevronDown)
    private let chevronUp = UIImage(resource: .iconChevronUp)

    var isDisabled = false {
        didSet {
            updateDisabled()
            updateBackgroundColor()
            updateStatusIndicatorColor()
        }
    }

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

        var contentMargins = UIMetrics.locationCellLayoutMargins
        contentMargins.leading += indentation

        contentView.directionalLayoutMargins = contentMargins
    }

    override func setHighlighted(_ highlighted: Bool, animated: Bool) {
        super.setHighlighted(highlighted, animated: animated)

        updateStatusIndicatorColor()
    }

    override func setSelected(_ selected: Bool, animated: Bool) {
        super.setSelected(selected, animated: animated)

        updateTickImage()
        updateStatusIndicatorColor()
    }

    private func setupCell() {
        indentationWidth = UIMetrics.TableView.cellIndentationWidth

        backgroundColor = .clear
        contentView.backgroundColor = .clear

        backgroundView = UIView()
        backgroundView?.backgroundColor = UIColor.Cell.Background.normal

        selectedBackgroundView = UIView()
        selectedBackgroundView?.backgroundColor = UIColor.Cell.Background.selected

        collapseButton.addTarget(self, action: #selector(handleCollapseButton(_:)), for: .touchUpInside)

        [locationLabel, tickImageView, statusIndicator, collapseButton].forEach { subview in
            subview.translatesAutoresizingMaskIntoConstraints = false
            contentView.addSubview(subview)
        }

        updateCollapseImage()
        updateAccessibilityCustomActions()
        updateDisabled()
        updateBackgroundColor()
        setLayoutMargins()

        NSLayoutConstraint.activate([
            tickImageView.leadingAnchor.constraint(equalTo: contentView.layoutMarginsGuide.leadingAnchor),
            tickImageView.centerYAnchor.constraint(equalTo: contentView.centerYAnchor),

            statusIndicator.widthAnchor.constraint(equalToConstant: 16),
            statusIndicator.heightAnchor.constraint(equalTo: statusIndicator.widthAnchor),
            statusIndicator.centerXAnchor.constraint(equalTo: tickImageView.centerXAnchor),
            statusIndicator.centerYAnchor.constraint(equalTo: tickImageView.centerYAnchor),

            locationLabel.leadingAnchor.constraint(
                equalTo: statusIndicator.trailingAnchor,
                constant: 12
            ),
            locationLabel.trailingAnchor.constraint(lessThanOrEqualTo: collapseButton.leadingAnchor)
                .withPriority(.defaultHigh),
            locationLabel.topAnchor.constraint(equalTo: contentView.layoutMarginsGuide.topAnchor),
            locationLabel.bottomAnchor.constraint(equalTo: contentView.layoutMarginsGuide.bottomAnchor),

            collapseButton.widthAnchor
                .constraint(
                    equalToConstant: UIMetrics.contentLayoutMargins.leading + UIMetrics
                        .contentLayoutMargins.trailing + 24
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
        backgroundView?.backgroundColor = backgroundColorForIdentationLevel()
        selectedBackgroundView?.backgroundColor = selectedBackgroundColorForIndentationLevel()
    }

    private func backgroundColorForIdentationLevel() -> UIColor {
        switch indentationLevel {
        case 1:
            return UIColor.Cell.Background.indentationLevelOne
        case 2:
            return UIColor.Cell.Background.indentationLevelTwo
        case 3:
            return UIColor.Cell.Background.indentationLevelThree
        default:
            return UIColor.Cell.Background.normal
        }
    }

    private func selectedBackgroundColorForIndentationLevel() -> UIColor {
        if isDisabled {
            return UIColor.Cell.Background.disabledSelected
        } else {
            return UIColor.Cell.Background.selected
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
        delegate?.toggle(cell: self)
    }

    @objc private func toggleCollapseAccessibilityAction() -> Bool {
        delegate?.toggle(cell: self)
        return true
    }

    private func updateCollapseImage() {
        let image = isExpanded ? chevronUp : chevronDown

        collapseButton.accessibilityIdentifier = isExpanded ? .collapseButton : .expandButton
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

extension LocationCell {
    func configureCell(item: LocationCellViewModel) {
        accessibilityIdentifier = item.node.code
        locationLabel.text = item.node.name
        showsCollapseControl = !item.node.children.isEmpty
        isExpanded = item.node.showsChildren
    }
}
