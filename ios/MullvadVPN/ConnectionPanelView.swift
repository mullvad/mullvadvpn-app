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

    let collapseButton: ConnectionPanelCollapseButton = {
        let button = ConnectionPanelCollapseButton(type: .custom)
        button.translatesAutoresizingMaskIntoConstraints = false
        button.tintColor = .white
        return button
    }()

    private let protocolRow = ConnectionPanelProtocolTypeRow()
    private let inAddressRow = ConnectionPanelAddressRow()
    private let outAddressRow = ConnectionPanelAddressRow()

    private lazy var stackView: UIStackView = {
        let stackView = UIStackView(arrangedSubviews: [protocolRow, inAddressRow, outAddressRow])
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
        commonInit()
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
        commonInit()
    }

    func didChangeDataSource() {
        inAddressRow.detailTextLabel.text = dataSource?.inAddress
        outAddressRow.detailTextLabel.text = dataSource?.outAddress
    }

    func toggleConnectionInfoVisibility() {
        showsConnectionInfo = !showsConnectionInfo
    }

    private func updateConnectionInfoVisibility() {
        stackView.isHidden = !showsConnectionInfo
        collapseButton.style = showsConnectionInfo ? .up : .down
    }

    private func commonInit() {
        protocolRow.translatesAutoresizingMaskIntoConstraints = false
        inAddressRow.translatesAutoresizingMaskIntoConstraints = false
        outAddressRow.translatesAutoresizingMaskIntoConstraints = false

        // TODO: Unhide it when we have out address
        outAddressRow.isHidden = true

        protocolRow.textLabel.text = NSLocalizedString("WireGuard", comment: "")
        inAddressRow.textLabel.text = NSLocalizedString("In", comment: "")
        outAddressRow.textLabel.text = NSLocalizedString("Out", comment: "")

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
                .constraint(equalTo: inAddressRow.textLabel.trailingAnchor),
            textLabelLayoutGuide.trailingAnchor
                .constraint(equalTo: outAddressRow.textLabel.trailingAnchor)
        ])

        updateConnectionInfoVisibility()
    }
}

class ConnectionPanelProtocolTypeRow: UIView {
    let textLabel = UILabel()

    override init(frame: CGRect) {
        super.init(frame: frame)

        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.font = UIFont.systemFont(ofSize: 17)
        textLabel.textColor = .white

        addSubview(textLabel)

        NSLayoutConstraint.activate([
            textLabel.topAnchor.constraint(equalTo: topAnchor),
            textLabel.bottomAnchor.constraint(equalTo: bottomAnchor),
            textLabel.leadingAnchor.constraint(equalTo: leadingAnchor),
            textLabel.trailingAnchor.constraint(equalTo: trailingAnchor)
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}

class ConnectionPanelAddressRow: UIView {
    let textLabel = UILabel()
    let detailTextLabel = UILabel()
    let stackView: UIStackView

    override init(frame: CGRect) {

        let font = UIFont.systemFont(ofSize: 17)

        textLabel.font = font
        textLabel.textColor = .white
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.setContentHuggingPriority(.defaultHigh, for: .horizontal)

        detailTextLabel.font = font
        detailTextLabel.textColor = .white
        detailTextLabel.translatesAutoresizingMaskIntoConstraints = false

        stackView = UIStackView(arrangedSubviews: [textLabel, detailTextLabel])
        stackView.spacing = UIStackView.spacingUseSystem
        stackView.translatesAutoresizingMaskIntoConstraints = false

        super.init(frame: frame)

        addSubview(stackView)

        NSLayoutConstraint.activate([
            stackView.topAnchor.constraint(equalTo: topAnchor),
            stackView.bottomAnchor.constraint(equalTo: bottomAnchor),
            stackView.leadingAnchor.constraint(equalTo: leadingAnchor),
            stackView.trailingAnchor.constraint(equalTo: trailingAnchor)
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}

class ConnectionPanelCollapseButton: CustomButton {

    enum Style {
        case up, down

        var image: UIImage {
            switch self {
            case .up:
                return UIImage(imageLiteralResourceName: "IconChevronUp")
            case .down:
                return UIImage(imageLiteralResourceName: "IconChevronDown")
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
