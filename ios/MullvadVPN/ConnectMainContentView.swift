//
//  ConnectMainContentView.swift
//  MullvadVPN
//
//  Created by pronebird on 09/03/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit
import MapKit

class ConnectMainContentView: UIView {
    enum ActionButton {
        case connect
        case disconnect
        case selectLocation
    }

    lazy var mapView: MKMapView = {
        let mapView = MKMapView()
        mapView.translatesAutoresizingMaskIntoConstraints = true
        mapView.autoresizingMask = [.flexibleWidth, .flexibleHeight]
        mapView.showsUserLocation = false
        mapView.isZoomEnabled = false
        mapView.isScrollEnabled = false
        mapView.isUserInteractionEnabled = false
        mapView.accessibilityElementsHidden = true
        return mapView
    }()

    let secureLabel = makeBoldTextLabel(ofSize: 20)
    let countryLabel = makeBoldTextLabel(ofSize: 34)
    let cityLabel = makeBoldTextLabel(ofSize: 34)

    lazy var connectionPanel: ConnectionPanelView = {
        let view = ConnectionPanelView()
        view.translatesAutoresizingMaskIntoConstraints = false
        return view
    }()

    lazy var buttonsStackView: UIStackView = {
        let stackView = UIStackView()
        stackView.spacing = 16
        stackView.axis = .vertical
        stackView.translatesAutoresizingMaskIntoConstraints = false
        return stackView
    }()

    lazy var connectButton: AppButton = {
        let button = AppButton(style: .success)
        button.translatesAutoresizingMaskIntoConstraints = false
        return button
    }()

    lazy var selectLocationButton: AppButton = {
        let button = AppButton(style: .translucentNeutral)
        button.accessibilityIdentifier = "SelectLocationButton"
        button.translatesAutoresizingMaskIntoConstraints = false
        return button
    }()

    let splitDisconnectButton: DisconnectSplitButton = {
        let button = DisconnectSplitButton()
        button.primaryButton.accessibilityIdentifier = "DisconnectButton"
        button.translatesAutoresizingMaskIntoConstraints = false
        return button
    }()

    let containerView: UIView = {
        let view = UIView()
        view.translatesAutoresizingMaskIntoConstraints = false
        return view
    }()

    private var traitConstraints = [NSLayoutConstraint]()

    override init(frame: CGRect) {
        super.init(frame: frame)

        backgroundColor = .primaryColor
        layoutMargins = UIMetrics.contentLayoutMargins

        addSubviews()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func setActionButtons(_ actionButtons: [ActionButton]) {
        let views = actionButtons.map { self.view(forActionButton: $0) }

        setArrangedButtons(views)
    }

    private class func makeBoldTextLabel(ofSize fontSize: CGFloat) -> UILabel {
        let textLabel = UILabel()
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.font = UIFont.boldSystemFont(ofSize: fontSize)
        textLabel.textColor = .white
        return textLabel
    }

    private func addSubviews() {
        mapView.frame = self.bounds
        addSubview(mapView)

        addSubview(containerView)
        [secureLabel, countryLabel, cityLabel, connectionPanel, buttonsStackView].forEach { containerView.addSubview($0) }

        NSLayoutConstraint.activate([
            containerView.topAnchor.constraint(equalTo: layoutMarginsGuide.topAnchor),
            containerView.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor),
            containerView.bottomAnchor.constraint(equalTo: layoutMarginsGuide.bottomAnchor),

            secureLabel.topAnchor.constraint(greaterThanOrEqualTo: containerView.topAnchor),
            secureLabel.leadingAnchor.constraint(equalTo: containerView.leadingAnchor),
            secureLabel.trailingAnchor.constraint(equalTo: containerView.trailingAnchor),

            countryLabel.topAnchor.constraint(equalTo: secureLabel.bottomAnchor, constant: 8),
            countryLabel.leadingAnchor.constraint(equalTo: containerView.leadingAnchor),
            countryLabel.trailingAnchor.constraint(equalTo: containerView.trailingAnchor),

            cityLabel.topAnchor.constraint(equalTo: countryLabel.bottomAnchor, constant: 8),
            cityLabel.leadingAnchor.constraint(equalTo: containerView.leadingAnchor),
            cityLabel.trailingAnchor.constraint(equalTo: containerView.trailingAnchor),

            connectionPanel.topAnchor.constraint(equalTo: cityLabel.bottomAnchor, constant: 8),
            connectionPanel.leadingAnchor.constraint(equalTo: containerView.leadingAnchor),
            connectionPanel.trailingAnchor.constraint(equalTo: containerView.trailingAnchor),

            buttonsStackView.topAnchor.constraint(equalTo: connectionPanel.bottomAnchor, constant: 24),
            buttonsStackView.leadingAnchor.constraint(equalTo: containerView.leadingAnchor),
            buttonsStackView.trailingAnchor.constraint(equalTo: containerView.trailingAnchor),
            buttonsStackView.bottomAnchor.constraint(equalTo: containerView.bottomAnchor)
        ])

        updateTraitConstraints()
    }

    private func updateTraitConstraints() {
        var layoutConstraints = [NSLayoutConstraint]()

        switch traitCollection.userInterfaceIdiom {
        case .pad:
            // Max container width is 70% width of iPad in portrait mode
            let maxWidth = min(UIScreen.main.nativeBounds.width * 0.7, UIMetrics.maximumSplitViewContentContainerWidth)
            let containerWidthConstraint = containerView.widthAnchor.constraint(equalToConstant: maxWidth)
            containerWidthConstraint.priority = .defaultHigh

            layoutConstraints.append(contentsOf:[
                containerView.trailingAnchor.constraint(lessThanOrEqualTo: layoutMarginsGuide.trailingAnchor),
                containerWidthConstraint
            ])

        case .phone:
            layoutConstraints.append(containerView.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor))

        default:
            break
        }

        traitConstraints = layoutConstraints
        removeConstraints(traitConstraints)
        NSLayoutConstraint.activate(layoutConstraints)
    }

    override func traitCollectionDidChange(_ previousTraitCollection: UITraitCollection?) {
        super.traitCollectionDidChange(previousTraitCollection)

        if traitCollection.userInterfaceIdiom != previousTraitCollection?.userInterfaceIdiom {
            updateTraitConstraints()
        }
    }

    private func setArrangedButtons(_ newButtons: [UIView]) {
        buttonsStackView.arrangedSubviews.forEach { (button) in
            if !newButtons.contains(button) {
                buttonsStackView.removeArrangedSubview(button)
                button.removeFromSuperview()
            }
        }

        newButtons.forEach { (button) in
            buttonsStackView.addArrangedSubview(button)
        }
    }

    private func view(forActionButton actionButton: ActionButton) -> UIView {
        switch actionButton {
        case .connect:
            return connectButton
        case .disconnect:
            return splitDisconnectButton
        case .selectLocation:
            return selectLocationButton
        }
    }
}
