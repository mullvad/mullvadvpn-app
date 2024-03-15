//
//  AddLocationCell.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-02-29.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import UIKit

protocol AddLocationCellDelegate: AnyObject {
    func toggleExpanding(cell: AddLocationCell)
    func toggleSelection(cell: AddLocationCell)
}

class AddLocationCell: UITableViewCell {
    weak var delegate: AddLocationCellDelegate?

    private let chevronDown = UIImage(resource: .iconChevronDown)
    private let chevronUp = UIImage(resource: .iconChevronUp)

    private let locationLabel: UILabel = {
        let label = UILabel()
        label.font = UIFont.systemFont(ofSize: 17)
        label.textColor = .white
        label.numberOfLines = .zero
        label.lineBreakStrategy = []
        return label
    }()

    private let checkboxButton: UIButton = {
        let button = UIButton()
        button.setImage(UIImage(systemName: "checkmark.square.fill"), for: .selected)
        button.setImage(UIImage(systemName: "square"), for: .normal)
        button.tintColor = .white
        return button
    }()

    private let collapseButton: UIButton = {
        let button = UIButton(type: .custom)
        button.accessibilityIdentifier = .collapseButton
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

    private func setupCell() {
        indentationWidth = UIMetrics.TableView.cellIndentationWidth

        backgroundColor = .clear
        contentView.backgroundColor = .clear

        backgroundView = UIView()

        collapseButton.addTarget(self, action: #selector(handleCollapseButton(_:)), for: .touchUpInside)
        checkboxButton.addTarget(self, action: #selector(toggleCheckboxButton(_:)), for: .touchUpInside)

        contentView.addConstrainedSubviews([checkboxButton, locationLabel, collapseButton]) {
            checkboxButton.pinEdgeToSuperviewMargin(.leading(.zero))
            checkboxButton.centerYAnchor.constraint(equalTo: contentView.centerYAnchor)
            checkboxButton.widthAnchor
                .constraint(
                    equalToConstant: 44.0
                )
            checkboxButton.heightAnchor.constraint(equalTo: checkboxButton.widthAnchor, multiplier: 1, constant: 0)

            locationLabel.leadingAnchor.constraint(
                equalTo: checkboxButton.trailingAnchor,
                constant: 12
            )

            locationLabel.trailingAnchor.constraint(lessThanOrEqualTo: collapseButton.leadingAnchor)
                .withPriority(.defaultHigh)
            locationLabel.pinEdgesToSuperviewMargins(PinnableEdges([.top(.zero), .bottom(.zero)]))

            collapseButton.widthAnchor
                .constraint(
                    equalToConstant: UIMetrics.contentLayoutMargins.leading + UIMetrics
                        .contentLayoutMargins.trailing + 24.0
                )
            collapseButton.pinEdgesToSuperviewMargins(.all().excluding(.leading))
        }

        updateCollapseImage()
        updateAccessibilityCustomActions()
        updateBackgroundColor()
        setLayoutMargins()
    }

    private func updateBackgroundColor() {
        backgroundView?.backgroundColor = backgroundColorForIdentationLevel()
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

    private func updateCollapseImage() {
        let image = isExpanded ? chevronUp : chevronDown
        collapseButton.setImage(image, for: .normal)
    }

    private func updateAccessibilityCustomActions() {
        if showsCollapseControl {
            let actionName = isExpanded
                ? NSLocalizedString(
                    "ADD_LOCATIONS_COLLAPSE_ACCESSIBILITY_ACTION",
                    tableName: "AddLocationsLocation",
                    value: "Collapse location",
                    comment: ""
                )
                : NSLocalizedString(
                    "ADD_LOCATIONS_EXPAND_ACCESSIBILITY_ACTION",
                    tableName: "AddLocationsLocation",
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

    // MARK: - actions

    @objc private func handleCollapseButton(_ sender: UIControl) {
        delegate?.toggleExpanding(cell: self)
    }

    @objc private func toggleCollapseAccessibilityAction() -> Bool {
        delegate?.toggleExpanding(cell: self)
        return true
    }

    @objc private func toggleCheckboxButton(_ sender: UIControl) {
        delegate?.toggleSelection(cell: self)
    }
}

extension AddLocationCell {
    func configure(item: AddLocationCellViewModel) {
        accessibilityIdentifier = item.node.name
        locationLabel.text = item.node.name
        showsCollapseControl = !item.node.children.isEmpty
        isExpanded = item.node.showsChildren
        checkboxButton.isSelected = item.isSelected
        checkboxButton.tintColor = item.isSelected ? .successColor : .white
    }
}
