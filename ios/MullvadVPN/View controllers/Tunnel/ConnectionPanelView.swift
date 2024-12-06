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
            collapseView.setAccessibilityIdentifier(.relayStatusCollapseButton)
            collapseView.title.text = connectedRelayName
            collapseView.accessibilityLabel = NSLocalizedString(
                "RELAY_ACCESSIBILITY_LABEL",
                tableName: "ConnectionPanel",
                value: "Connected relay",
                comment: ""
            )
            collapseView.accessibilityAttributedValue = NSAttributedString(
                string: connectedRelayName.replacingOccurrences(
                    of: "-wireguard",
                    with: " WireGuard"
                ),
                attributes: [.accessibilitySpeechLanguage: "en"]
            )
        }
    }

    private let collapseView: ConnectionPanelCollapseView = {
        let collapseView = ConnectionPanelCollapseView()
        collapseView.axis = .horizontal
        collapseView.alignment = .top
        collapseView.distribution = .fill
        collapseView.translatesAutoresizingMaskIntoConstraints = false
        collapseView.tintColor = .white
        collapseView.isAccessibilityElement = false
        return collapseView
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

        inAddressRow.setAccessibilityIdentifier(.connectionPanelInAddressRow)
        outAddressRow.setAccessibilityIdentifier(.connectionPanelOutAddressRow)

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

        addSubview(collapseView)
        addSubview(stackView)
        addLayoutGuide(textLabelLayoutGuide)

        NSLayoutConstraint.activate([
            collapseView.topAnchor.constraint(equalTo: topAnchor),
            collapseView.leadingAnchor.constraint(equalTo: leadingAnchor),
            collapseView.trailingAnchor.constraint(equalTo: trailingAnchor),

            stackView.topAnchor.constraint(equalTo: collapseView.bottomAnchor, constant: 4),
            stackView.leadingAnchor.constraint(equalTo: leadingAnchor),
            stackView.trailingAnchor.constraint(equalTo: trailingAnchor),
            stackView.bottomAnchor.constraint(equalTo: bottomAnchor),

            inAddressRow.heightAnchor.constraint(equalToConstant: UIMetrics.ConnectionPanelView.inRowHeight),
            outAddressRow.heightAnchor.constraint(equalToConstant: UIMetrics.ConnectionPanelView.outRowHeight),

            // Align all text labels with the guide, so that they maintain equal width
            textLabelLayoutGuide.trailingAnchor
                .constraint(equalTo: inAddressRow.textLabelLayoutGuide.trailingAnchor),
            textLabelLayoutGuide.trailingAnchor
                .constraint(equalTo: outAddressRow.textLabelLayoutGuide.trailingAnchor),
        ])

        updateConnectionInfoVisibility()
        updateCollapseButtonAccessibilityHint()

        let longPressGestureRecognizer = UILongPressGestureRecognizer(
            target: self,
            action: #selector(toggleCollapse(_:))
        )
        longPressGestureRecognizer.minimumPressDuration = 0
        collapseView.addGestureRecognizer(longPressGestureRecognizer)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func didChangeDataSource() {
        inAddressRow.value = dataSource?.inAddress
        inAddressRow.alpha = dataSource?.inAddress == nil ? 0 : 1.0

        outAddressRow.value = dataSource?.outAddress
        outAddressRow.alpha = dataSource?.outAddress == nil ? 0 : 1.0
    }

    private func toggleConnectionInfoVisibility() {
        showsConnectionInfo = !showsConnectionInfo
    }

    @objc private func toggleCollapse(_ sender: UILongPressGestureRecognizer) {
        switch sender.state {
        case .began:
            collapseView.title.textColor = .lightGray
            collapseView.imageView.tintColor = .lightGray
        case .ended:
            collapseView.title.textColor = .white
            collapseView.imageView.tintColor = .white
            toggleConnectionInfoVisibility()
        default:
            break
        }
    }

    private func updateConnectionInfoVisibility() {
        stackView.isHidden = !showsConnectionInfo
        collapseView.style = showsConnectionInfo ? .up : .down

        if collapseView.accessibilityElementIsFocused(), showsConnectionInfo {
            UIAccessibility.post(
                notification: .layoutChanged,
                argument: stackView.arrangedSubviews.first
            )
        }
        updateCollapseButtonAccessibilityHint()
    }

    private func updateCollapseButtonAccessibilityHint() {
        if showsConnectionInfo {
            collapseView.accessibilityHint = NSLocalizedString(
                "COLLAPSE_BUTTON_ACCESSIBILITY_HINT",
                tableName: "ConnectionPanel",
                value: "Double tap to collapse the connection info panel.",
                comment: ""
            )
        } else {
            collapseView.accessibilityHint = NSLocalizedString(
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
        detailTextLabel.setAccessibilityIdentifier(.connectionPanelDetailLabel)
        detailTextLabel.font = .systemFont(ofSize: 17)
        detailTextLabel.textColor = .white
        detailTextLabel.translatesAutoresizingMaskIntoConstraints = false
        detailTextLabel.numberOfLines = .zero
        detailTextLabel.lineBreakMode = .byWordWrapping
        return detailTextLabel
    }()

    private lazy var stackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [textLabel, detailTextLabel])
        stackView.spacing = UIStackView.spacingUseSystem
        stackView.translatesAutoresizingMaskIntoConstraints = false
        stackView.alignment = .top
        return stackView
    }()

    let textLabelLayoutGuide = UILayoutGuide()

    var title: String? {
        get {
            textLabel.text
        }
        set {
            textLabel.text = newValue
            accessibilityLabel = newValue
        }
    }

    var value: String? {
        get {
            detailTextLabel.text
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

class ConnectionPanelCollapseView: UIStackView {
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
            updateImage()
        }
    }

    private(set) var title: UILabel = {
        let button = UILabel()
        button.textColor = .white
        button.numberOfLines = 0
        return button
    }()

    private(set) var imageView: UIImageView = {
        let imageView = UIImageView()
        imageView.contentMode = .scaleAspectFit
        return imageView
    }()

    override init(frame: CGRect) {
        super.init(frame: frame)

        addArrangedSubview(title)
        addArrangedSubview(imageView)

        title.setContentHuggingPriority(.defaultLow, for: .horizontal)
        title.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)

        imageView.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        imageView.setContentCompressionResistancePriority(.defaultHigh, for: .horizontal)
        addArrangedSubview(UIView()) // Pushes content left.

        updateImage()
    }

    required init(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func updateImage() {
        imageView.image = style.image
    }
}
