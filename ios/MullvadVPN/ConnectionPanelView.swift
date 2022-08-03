//
//  ConnectionPanelView.swift
//  MullvadVPN
//
//  Created by pronebird on 12/02/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

struct ConnectionPanelData {
    var inAddress: String
    var outAddress: String?
}

class ConnectionPanelView: UIView {
    var dataSource: ConnectionPanelData? {
        didSet {
            didChangeDataSource()
        }
    }

    var showsConnectionInfo = false {
        didSet {
            updateConnectionInfoVisibility()
        }
    }

    var connectedRelayName = "" {
        didSet {
            collapseButton.setTitle(connectedRelayName, for: .normal)
            collapseButton.accessibilityLabel = NSLocalizedString(
                "RELAY_ACCESSIBILITY_LABEL",
                tableName: "ConnectionPanel",
                value: "Connected relay",
                comment: ""
            )
            collapseButton.accessibilityAttributedValue = NSAttributedString(
                string: connectedRelayName.replacingOccurrences(
                    of: "-wireguard",
                    with: " WireGuard"
                ),
                attributes: [.accessibilitySpeechLanguage: "en"]
            )
        }
    }

    private let collapseButton: ConnectionPanelCollapseButton = {
        let button = ConnectionPanelCollapseButton(type: .custom)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.tintColor = .white
        return button
    }()

    private let inAddressRow = ConnectionPanelAddressRow()
    private let outAddressRow = ConnectionPanelAddressRow()

    private lazy var stackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [inAddressRow, outAddressRow])
        stackView.axis = .vertical
        stackView.translatesAutoresizingMaskIntoConstraints = false
        return stackView
    }()

    private let textLabelLayoutGuide: UILayoutGuide = {
        let layoutGuide = UILayoutGuide()
        layoutGuide.identifier = "TextLabelLayoutGuide"
        return layoutGuide
    }()

    override init(frame: CGRect) {
        super.init(frame: frame)

        inAddressRow.translatesAutoresizingMaskIntoConstraints = false
        outAddressRow.translatesAutoresizingMaskIntoConstraints = false

        // TODO: Unhide it when we have out address
        outAddressRow.isHidden = true

        inAddressRow.title = NSLocalizedString(
            "IN_ADDRESS_LABEL",
            tableName: "ConnectionPanel",
            value: "In",
            comment: ""
        )
        outAddressRow.title = NSLocalizedString(
            "OUT_ADDRESS_LABEL",
            tableName: "ConnectionPanel",
            value: "Out",
            comment: ""
        )

        addSubview(collapseButton)
        addSubview(stackView)
        addLayoutGuide(textLabelLayoutGuide)

        NSLayoutConstraint.activate([
            collapseButton.topAnchor.constraint(equalTo: topAnchor),
            collapseButton.leadingAnchor.constraint(equalTo: leadingAnchor),
            collapseButton.trailingAnchor.constraint(equalTo: trailingAnchor),

            stackView.topAnchor.constraint(equalTo: collapseButton.bottomAnchor, constant: 4),
            stackView.leadingAnchor.constraint(equalTo: leadingAnchor),
            stackView.trailingAnchor.constraint(equalTo: trailingAnchor),
            stackView.bottomAnchor.constraint(equalTo: bottomAnchor),

            // Align all text labels with the guide, so that they maintain equal width
            textLabelLayoutGuide.trailingAnchor
                .constraint(equalTo: inAddressRow.textLabelLayoutGuide.trailingAnchor),
            textLabelLayoutGuide.trailingAnchor
                .constraint(equalTo: outAddressRow.textLabelLayoutGuide.trailingAnchor),
        ])

        updateConnectionInfoVisibility()
        updateCollapseButtonAccessibilityHint()

        collapseButton.addTarget(self, action: #selector(toggleCollapse(_:)), for: .touchUpInside)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func didChangeDataSource() {
        inAddressRow.value = dataSource?.inAddress
        outAddressRow.value = dataSource?.outAddress
    }

    private func toggleConnectionInfoVisibility() {
        showsConnectionInfo = !showsConnectionInfo
    }

    @objc private func toggleCollapse(_ sender: Any) {
        toggleConnectionInfoVisibility()
    }

    private func updateConnectionInfoVisibility() {
        stackView.isHidden = !showsConnectionInfo
        collapseButton.style = showsConnectionInfo ? .up : .down

        if collapseButton.accessibilityElementIsFocused(), showsConnectionInfo {
            UIAccessibility.post(
                notification: .layoutChanged,
                argument: stackView.arrangedSubviews.first
            )
        }
        updateCollapseButtonAccessibilityHint()
    }

    private func updateCollapseButtonAccessibilityHint() {
        if showsConnectionInfo {
            collapseButton.accessibilityHint = NSLocalizedString(
                "COLLAPSE_BUTTON_ACCESSIBILITY_HINT",
                tableName: "ConnectionPanel",
                value: "Double tap to collapse the connection info panel.",
                comment: ""
            )
        } else {
            collapseButton.accessibilityHint = NSLocalizedString(
                "EXPAND_BUTTON_ACCESSIBILITY_HINT",
                tableName: "ConnectionPanel",
                value: "Double tap to expand the connection info panel.",
                comment: ""
            )
        }
    }
}

class ConnectionPanelAddressRow: UIView {
    private let textLabel: UILabel = {
        let textLabel = UILabel()
        textLabel.font = .systemFont(ofSize: 17)
        textLabel.textColor = .white
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        return textLabel
    }()

    private let detailTextLabel: UILabel = {
        let detailTextLabel = UILabel()
        detailTextLabel.font = .systemFont(ofSize: 17)
        detailTextLabel.textColor = .white
        detailTextLabel.translatesAutoresizingMaskIntoConstraints = false
        return detailTextLabel
    }()

    private lazy var stackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [textLabel, detailTextLabel])
        stackView.spacing = UIStackView.spacingUseSystem
        stackView.translatesAutoresizingMaskIntoConstraints = false
        return stackView
    }()

    let textLabelLayoutGuide = UILayoutGuide()

    var title: String? {
        get {
            return textLabel.text
        }
        set {
            textLabel.text = newValue
            accessibilityLabel = newValue
        }
    }

    var value: String? {
        get {
            return detailTextLabel.text
        }
        set {
            detailTextLabel.text = newValue
            accessibilityValue = newValue
        }
    }

    override init(frame: CGRect) {
        super.init(frame: frame)

        isAccessibilityElement = true

        addSubview(stackView)
        addLayoutGuide(textLabelLayoutGuide)

        NSLayoutConstraint.activate([
            stackView.topAnchor.constraint(equalTo: topAnchor),
            stackView.bottomAnchor.constraint(equalTo: bottomAnchor),
            stackView.leadingAnchor.constraint(equalTo: leadingAnchor),
            stackView.trailingAnchor.constraint(equalTo: trailingAnchor),

            textLabelLayoutGuide.leadingAnchor.constraint(equalTo: textLabel.leadingAnchor),
            textLabelLayoutGuide.trailingAnchor.constraint(equalTo: textLabel.trailingAnchor),
            textLabelLayoutGuide.topAnchor.constraint(equalTo: textLabel.topAnchor),
            textLabelLayoutGuide.bottomAnchor.constraint(equalTo: textLabel.bottomAnchor),
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}

class ConnectionPanelCollapseButton: CustomButton {
    enum Style {
        case up, down

        var image: UIImage? {
            switch self {
            case .up:
                return UIImage(named: "IconChevronUp")
            case .down:
                return UIImage(named: "IconChevronDown")
            }
        }
    }

    var style = Style.up {
        didSet {
            updateButtonImage()
        }
    }

    override init(frame: CGRect) {
        super.init(frame: frame)
        commonInit()
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
        commonInit()
    }

    private func commonInit() {
        setTitleColor(UIColor.white, for: .normal)
        setTitleColor(UIColor.lightGray, for: .highlighted)
        setTitleColor(UIColor.lightGray, for: .disabled)

        contentHorizontalAlignment = .leading
        imageAlignment = .trailing
        inlineImageSpacing = 0

        updateButtonImage()
    }

    private func updateButtonImage() {
        setImage(style.image, for: .normal)
    }
}
