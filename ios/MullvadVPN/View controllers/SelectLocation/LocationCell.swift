//
//  LocationCell.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

protocol LocationCellDelegate: AnyObject {
    func toggleExpanding(cell: LocationCell)
    func toggleSelecting(cell: LocationCell)
}

class LocationCell: UITableViewCell {
    weak var delegate: LocationCellDelegate?

    private let locationLabel: UILabel = {
        let label = UILabel()
        label.font = UIFont.systemFont(ofSize: 16)
        label.textColor = .white
        label.lineBreakMode = .byTruncatingTail
        label.numberOfLines = 1
        return label
    }()

    private let statusIndicator: UIView = {
        let view = UIView()
        view.layer.cornerRadius = 8
        view.layer.cornerCurve = .circular
        return view
    }()

    private let tickImageView: UIImageView = {
        let imageView = UIImageView(image: UIImage.tick)
        imageView.tintColor = .white
        return imageView
    }()

    private let checkboxButton: UIButton = {
        let button = UIButton()
        let checkboxView = CheckboxView()

        checkboxView.isUserInteractionEnabled = false
        button.addConstrainedSubviews([checkboxView]) {
            checkboxView.pinEdgesToSuperviewMargins(PinnableEdges([.top(8), .bottom(8), .leading(16), .trailing(16)]))
        }

        return button
    }()

    private let collapseButton: UIButton = {
        let button = UIButton(type: .custom)
        button.isAccessibilityElement = false
        button.tintColor = .white
        return button
    }()

    private var locationLabelLeadingMargin: CGFloat {
        switch behavior {
        case .add:
            0
        case .select:
            12
        }
    }

    private var behavior: LocationCellBehavior = .select
    private let chevronDown = UIImage.CellDecoration.chevronDown
    private let chevronUp = UIImage.CellDecoration.chevronUp

    var isDisabled = false {
        didSet {
            updateDisabled(isDisabled)
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

        updateLeadingImage()
        updateStatusIndicatorColor()
    }

    private func setupCell() {
        indentationWidth = UIMetrics.TableView.cellIndentationWidth

        backgroundColor = .clear
        contentView.backgroundColor = .clear

        backgroundView = UIView()
        selectedBackgroundView = UIView()

        checkboxButton.addTarget(self, action: #selector(toggleCheckboxButton(_:)), for: .touchUpInside)
        collapseButton.addTarget(self, action: #selector(handleCollapseButton(_:)), for: .touchUpInside)

        [locationLabel, tickImageView, statusIndicator, collapseButton].forEach { subview in
            subview.translatesAutoresizingMaskIntoConstraints = false
            contentView.addSubview(subview)
        }

        updateCollapseImage()
        updateAccessibilityCustomActions()
        updateDisabled(isDisabled)
        updateBackgroundColor()
        setLayoutMargins()

        contentView.addConstrainedSubviews([
            tickImageView,
            statusIndicator,
            locationLabel,
            collapseButton,
            checkboxButton,
        ]) {
            tickImageView.pinEdgesToSuperviewMargins(PinnableEdges([.leading(0)]))
            tickImageView.centerYAnchor.constraint(equalTo: contentView.centerYAnchor)

            statusIndicator.widthAnchor.constraint(equalToConstant: 16)
            statusIndicator.heightAnchor.constraint(equalTo: statusIndicator.widthAnchor)
            statusIndicator.centerXAnchor.constraint(equalTo: tickImageView.centerXAnchor)
            statusIndicator.centerYAnchor.constraint(equalTo: tickImageView.centerYAnchor)

            checkboxButton.pinEdgesToSuperview(PinnableEdges([.top(0), .bottom(0)]))
            checkboxButton.trailingAnchor.constraint(equalTo: locationLabel.leadingAnchor, constant: 14)
            checkboxButton.widthAnchor.constraint(
                equalToConstant: UIMetrics.contentLayoutMargins.leading + UIMetrics.contentLayoutMargins.trailing + 24
            )

            locationLabel.pinEdgesToSuperviewMargins(PinnableEdges([.top(0), .bottom(0)]))
            locationLabel.leadingAnchor.constraint(
                equalTo: statusIndicator.trailingAnchor,
                constant: locationLabelLeadingMargin
            )
            locationLabel.trailingAnchor.constraint(lessThanOrEqualTo: collapseButton.leadingAnchor)
                .withPriority(.defaultHigh)

            collapseButton.widthAnchor.constraint(
                equalToConstant: UIMetrics.contentLayoutMargins.leading + UIMetrics.contentLayoutMargins.trailing + 24
            )
            collapseButton.pinEdgesToSuperview(.all().excluding(.leading))
        }
    }

    private func updateLeadingImage() {
        switch behavior {
        case .add:
            checkboxButton.isHidden = false
            statusIndicator.isHidden = true
            tickImageView.isHidden = true
        case .select:
            checkboxButton.isHidden = true
            statusIndicator.isHidden = isSelected
            tickImageView.isHidden = !isSelected
        }
    }

    private func updateStatusIndicatorColor() {
        statusIndicator.backgroundColor = statusIndicatorColor()
    }

    private func updateDisabled(_ isDisabled: Bool) {
        locationLabel.alpha = isDisabled ? 0.2 : 1

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
        delegate?.toggleExpanding(cell: self)
    }

    @objc private func toggleCollapseAccessibilityAction() -> Bool {
        delegate?.toggleExpanding(cell: self)
        return true
    }

    private func updateCollapseImage() {
        let image = isExpanded ? chevronUp : chevronDown

        collapseButton.setAccessibilityIdentifier(isExpanded ? .collapseButton : .expandButton)
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

    @objc private func toggleCheckboxButton(_ sender: UIControl) {
        delegate?.toggleSelecting(cell: self)
    }
}

extension LocationCell {
    enum LocationCellBehavior {
        case add
        case select
    }

    func configure(item: LocationCellViewModel, behavior: LocationCellBehavior) {
        isDisabled = !item.node.isActive
        locationLabel.text = item.node.name
        showsCollapseControl = !item.node.children.isEmpty
        isExpanded = item.node.showsChildren
        accessibilityValue = item.node.code
        checkboxButton.setAccessibilityIdentifier(.customListLocationCheckmarkButton)

        for view in checkboxButton.subviews where view is CheckboxView {
            let checkboxView = view as? CheckboxView
            checkboxView?.isChecked = item.isSelected
        }

        if item.node is CustomListLocationNode {
            setAccessibilityIdentifier(.customListLocationCell)
        } else {
            // Only custom list nodes have more than one location. Therefore checking first
            // location here is fine.
            if let location = item.node.locations.first {
                let accessibilityId: AccessibilityIdentifier = switch location {
                case .country: .countryLocationCell
                case .city: .cityLocationCell
                case .hostname: .relayLocationCell
                }
                setAccessibilityIdentifier(accessibilityId)
            }
        }

        setBehavior(behavior)
    }

    func setExcluded(relayTitle: String? = nil) {
        updateDisabled(true)

        if let relayTitle {
            locationLabel.text! += " (\(relayTitle))"
        }
    }

    private func setBehavior(_ newBehavior: LocationCellBehavior) {
        self.behavior = newBehavior
        updateLeadingImage()
    }
}
