//
//  TunnelControlView.swift
//  MullvadVPN
//
//  Created by pronebird on 09/03/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import MapKit
import MullvadTypes
import UIKit

enum TunnelControlAction {
    case connect
    case disconnect
    case cancel
    case reconnect
    case selectLocation
}

private enum TunnelControlActionButton {
    case connect
    case disconnect
    case cancel
    case selectLocation
}

final class TunnelControlView: UIView {
    private let secureLabel = makeBoldTextLabel(ofSize: 20)
    private let cityLabel = makeBoldTextLabel(ofSize: 34)
    private let countryLabel = makeBoldTextLabel(ofSize: 34)

    private let activityIndicator: SpinnerActivityIndicatorView = {
        let activityIndicator = SpinnerActivityIndicatorView(style: .large)
        activityIndicator.translatesAutoresizingMaskIntoConstraints = false
        activityIndicator.tintColor = .white
        activityIndicator.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        activityIndicator.setContentCompressionResistancePriority(.defaultHigh, for: .horizontal)
        return activityIndicator
    }()

    private let locationContainerView: UIView = {
        let view = UIView()
        view.translatesAutoresizingMaskIntoConstraints = false
        view.isAccessibilityElement = true
        view.accessibilityTraits = .summaryElement
        return view
    }()

    private let connectionPanel: ConnectionPanelView = {
        let view = ConnectionPanelView()
        view.translatesAutoresizingMaskIntoConstraints = false
        return view
    }()

    private let buttonsStackView: UIStackView = {
        let stackView = UIStackView()
        stackView.spacing = UIMetrics.interButtonSpacing
        stackView.axis = .vertical
        stackView.translatesAutoresizingMaskIntoConstraints = false
        return stackView
    }()

    private let connectButton: AppButton = {
        let button = AppButton(style: .success)
        button.translatesAutoresizingMaskIntoConstraints = false
        return button
    }()

    private let cancelButton: AppButton = {
        let button = AppButton(style: .translucentDanger)
        button.accessibilityIdentifier = "CancelButton"
        button.translatesAutoresizingMaskIntoConstraints = false
        return button
    }()

    private let selectLocationButton: AppButton = {
        let button = AppButton(style: .translucentNeutral)
        button.accessibilityIdentifier = "SelectLocationButton"
        button.translatesAutoresizingMaskIntoConstraints = false
        return button
    }()

    private let selectLocationBlurView: TranslucentButtonBlurView
    private let cancelButtonBlurView: TranslucentButtonBlurView

    private let splitDisconnectButton: DisconnectSplitButton = {
        let button = DisconnectSplitButton()
        button.primaryButton.accessibilityIdentifier = "DisconnectButton"
        button.translatesAutoresizingMaskIntoConstraints = false
        return button
    }()

    private let containerView: UIView = {
        let view = UIView()
        view.translatesAutoresizingMaskIntoConstraints = false
        return view
    }()

    private var traitConstraints = [NSLayoutConstraint]()
    private var tunnelState: TunnelState = .disconnected

    var actionHandler: ((TunnelControlAction) -> Void)?

    var mapCenterAlignmentView: UIView {
        return activityIndicator
    }

    override init(frame: CGRect) {
        selectLocationBlurView = TranslucentButtonBlurView(button: selectLocationButton)
        cancelButtonBlurView = TranslucentButtonBlurView(button: cancelButton)

        super.init(frame: frame)

        backgroundColor = .clear
        layoutMargins = UIMetrics.contentLayoutMargins
        accessibilityContainerType = .semanticGroup

        addSubviews()
        addButtonHandlers()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func traitCollectionDidChange(_ previousTraitCollection: UITraitCollection?) {
        super.traitCollectionDidChange(previousTraitCollection)

        if traitCollection.userInterfaceIdiom != previousTraitCollection?.userInterfaceIdiom {
            updateTraitConstraints()
        }

        if previousTraitCollection?.userInterfaceIdiom != traitCollection.userInterfaceIdiom ||
            previousTraitCollection?.horizontalSizeClass != traitCollection.horizontalSizeClass
        {
            updateActionButtons()
        }
    }

    func update(from tunnelState: TunnelState, animated: Bool) {
        self.tunnelState = tunnelState

        updateSecureLabel()
        updateActionButtons()
        updateTunnelRelay()
    }

    func setAnimatingActivity(_ isAnimating: Bool) {
        if isAnimating {
            activityIndicator.startAnimating()
        } else {
            activityIndicator.stopAnimating()
        }
    }

    private func updateActionButtons() {
        let actionButtons = tunnelState.actionButtons(traitCollection: traitCollection)
        let views = actionButtons.map { self.view(forActionButton: $0) }

        updateButtonTitles()
        setArrangedButtons(views)
    }

    private func updateSecureLabel() {
        secureLabel.text = tunnelState.localizedTitleForSecureLabel.uppercased()
        secureLabel.textColor = tunnelState.textColorForSecureLabel
    }

    private func updateButtonTitles() {
        connectButton.setTitle(
            NSLocalizedString(
                "CONNECT_BUTTON_TITLE",
                tableName: "Main",
                value: "Secure connection",
                comment: ""
            ), for: .normal
        )
        selectLocationButton.setTitle(
            tunnelState.localizedTitleForSelectLocationButton,
            for: .normal
        )
        cancelButton.setTitle(
            NSLocalizedString(
                "CANCEL_BUTTON_TITLE",
                tableName: "Main",
                value: "Cancel",
                comment: ""
            ), for: .normal
        )
        splitDisconnectButton.primaryButton.setTitle(
            NSLocalizedString(
                "DISCONNECT_BUTTON_TITLE",
                tableName: "Main",
                value: "Disconnect",
                comment: ""
            ), for: .normal
        )
        splitDisconnectButton.secondaryButton.accessibilityLabel = NSLocalizedString(
            "RECONNECT_BUTTON_ACCESSIBILITY_LABEL",
            tableName: "Main",
            value: "Reconnect",
            comment: ""
        )
    }

    private func updateTunnelRelay() {
        if let tunnelRelay = tunnelState.relay {
            cityLabel.attributedText = attributedStringForLocation(
                string: tunnelRelay.location.city
            )
            countryLabel.attributedText = attributedStringForLocation(
                string: tunnelRelay.location.country
            )

            connectionPanel.dataSource = ConnectionPanelData(
                inAddress: "\(tunnelRelay.ipv4Relay) UDP",
                outAddress: nil
            )
            connectionPanel.isHidden = false
            connectionPanel.connectedRelayName = tunnelRelay.hostname
        } else {
            countryLabel.attributedText = attributedStringForLocation(string: " ")
            cityLabel.attributedText = attributedStringForLocation(string: " ")
            connectionPanel.dataSource = nil
            connectionPanel.isHidden = true
        }

        locationContainerView.accessibilityLabel = tunnelState.localizedAccessibilityLabel
    }

    // MARK: - Private

    private func addSubviews() {
        for subview in [secureLabel, cityLabel, countryLabel] {
            locationContainerView.addSubview(subview)
        }

        for subview in [
            activityIndicator,
            locationContainerView,
            connectionPanel,
            buttonsStackView,
        ] {
            containerView.addSubview(subview)
        }

        addSubview(containerView)

        NSLayoutConstraint.activate([
            containerView.topAnchor.constraint(equalTo: layoutMarginsGuide.topAnchor),
            containerView.leadingAnchor.constraint(equalTo: layoutMarginsGuide.leadingAnchor),
            containerView.bottomAnchor.constraint(equalTo: layoutMarginsGuide.bottomAnchor),

            locationContainerView.topAnchor
                .constraint(greaterThanOrEqualTo: containerView.topAnchor),
            locationContainerView.leadingAnchor.constraint(equalTo: containerView.leadingAnchor),
            locationContainerView.trailingAnchor.constraint(equalTo: containerView.trailingAnchor),

            activityIndicator.centerXAnchor.constraint(equalTo: containerView.centerXAnchor),
            locationContainerView.topAnchor.constraint(
                equalTo: activityIndicator.bottomAnchor,
                constant: 22
            ),

            secureLabel.topAnchor.constraint(equalTo: locationContainerView.topAnchor),
            secureLabel.leadingAnchor.constraint(equalTo: locationContainerView.leadingAnchor),
            secureLabel.trailingAnchor.constraint(equalTo: locationContainerView.trailingAnchor),

            cityLabel.topAnchor.constraint(equalTo: secureLabel.bottomAnchor, constant: 8),
            cityLabel.leadingAnchor.constraint(equalTo: locationContainerView.leadingAnchor),
            cityLabel.trailingAnchor.constraint(equalTo: locationContainerView.trailingAnchor),

            countryLabel.topAnchor.constraint(equalTo: cityLabel.bottomAnchor, constant: 8),
            countryLabel.leadingAnchor.constraint(equalTo: locationContainerView.leadingAnchor),
            countryLabel.trailingAnchor.constraint(equalTo: locationContainerView.trailingAnchor),
            countryLabel.bottomAnchor.constraint(equalTo: locationContainerView.bottomAnchor),

            connectionPanel.topAnchor.constraint(
                equalTo: locationContainerView.bottomAnchor,
                constant: 8
            ),
            connectionPanel.leadingAnchor.constraint(equalTo: containerView.leadingAnchor),
            connectionPanel.trailingAnchor.constraint(equalTo: containerView.trailingAnchor),

            buttonsStackView.topAnchor.constraint(
                equalTo: connectionPanel.bottomAnchor,
                constant: 24
            ),
            buttonsStackView.leadingAnchor.constraint(equalTo: containerView.leadingAnchor),
            buttonsStackView.trailingAnchor.constraint(equalTo: containerView.trailingAnchor),
            buttonsStackView.bottomAnchor.constraint(equalTo: containerView.bottomAnchor),
        ])

        updateTraitConstraints()
    }

    private func addButtonHandlers() {
        connectButton.addTarget(
            self,
            action: #selector(handleConnect),
            for: .touchUpInside
        )
        cancelButton.addTarget(
            self,
            action: #selector(handleDisconnect),
            for: .touchUpInside
        )
        splitDisconnectButton.primaryButton.addTarget(
            self,
            action: #selector(handleDisconnect),
            for: .touchUpInside
        )
        splitDisconnectButton.secondaryButton.addTarget(
            self,
            action: #selector(handleReconnect),
            for: .touchUpInside
        )
        selectLocationButton.addTarget(
            self,
            action: #selector(handleSelectLocation),
            for: .touchUpInside
        )
    }

    private func updateTraitConstraints() {
        var layoutConstraints = [NSLayoutConstraint]()

        switch traitCollection.userInterfaceIdiom {
        case .pad:
            // Max container width is 70% width of iPad in portrait mode
            let maxWidth = min(
                UIScreen.main.nativeBounds.width * 0.7,
                UIMetrics.maximumSplitViewContentContainerWidth
            )

            layoutConstraints.append(contentsOf: [
                containerView.trailingAnchor.constraint(
                    lessThanOrEqualTo: layoutMarginsGuide.trailingAnchor
                ),
                containerView.widthAnchor.constraint(equalToConstant: maxWidth)
                    .withPriority(.defaultHigh),
            ])

        case .phone:
            layoutConstraints.append(
                containerView.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor)
            )

        default:
            break
        }

        removeConstraints(traitConstraints)
        traitConstraints = layoutConstraints
        NSLayoutConstraint.activate(layoutConstraints)
    }

    private func setArrangedButtons(_ newButtons: [UIView]) {
        buttonsStackView.arrangedSubviews.forEach { button in
            if !newButtons.contains(button) {
                buttonsStackView.removeArrangedSubview(button)
                button.removeFromSuperview()
            }
        }

        newButtons.forEach { button in
            buttonsStackView.addArrangedSubview(button)
        }
    }

    private func view(forActionButton actionButton: TunnelControlActionButton) -> UIView {
        switch actionButton {
        case .connect:
            return connectButton
        case .disconnect:
            return splitDisconnectButton
        case .cancel:
            return cancelButtonBlurView
        case .selectLocation:
            return selectLocationBlurView
        }
    }

    private func attributedStringForLocation(string: String) -> NSAttributedString {
        let paragraphStyle = NSMutableParagraphStyle()
        paragraphStyle.lineSpacing = 0
        paragraphStyle.lineHeightMultiple = 0.80

        return NSAttributedString(
            string: string,
            attributes: [.paragraphStyle: paragraphStyle]
        )
    }

    private class func makeBoldTextLabel(ofSize fontSize: CGFloat) -> UILabel {
        let textLabel = UILabel()
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.font = UIFont.boldSystemFont(ofSize: fontSize)
        textLabel.textColor = .white
        return textLabel
    }

    // MARK: - Actions

    @objc private func handleConnect() {
        actionHandler?(.connect)
    }

    @objc private func handleDisconnect() {
        actionHandler?(.disconnect)
    }

    @objc private func handleReconnect() {
        actionHandler?(.reconnect)
    }

    @objc private func handleSelectLocation() {
        actionHandler?(.selectLocation)
    }
}

private extension TunnelState {
    var textColorForSecureLabel: UIColor {
        switch self {
        case .connecting, .reconnecting, .waitingForConnectivity:
            return .white

        case .connected:
            return .successColor

        case .disconnecting, .disconnected, .pendingReconnect:
            return .dangerColor
        }
    }

    var localizedTitleForSecureLabel: String {
        switch self {
        case .connecting, .reconnecting:
            return NSLocalizedString(
                "TUNNEL_STATE_CONNECTING",
                tableName: "Main",
                value: "Creating secure connection",
                comment: ""
            )

        case .connected:
            return NSLocalizedString(
                "TUNNEL_STATE_CONNECTED",
                tableName: "Main",
                value: "Secure connection",
                comment: ""
            )

        case .disconnecting(.nothing):
            return NSLocalizedString(
                "TUNNEL_STATE_DISCONNECTING",
                tableName: "Main",
                value: "Disconnecting",
                comment: ""
            )
        case .disconnecting(.reconnect), .pendingReconnect:
            return NSLocalizedString(
                "TUNNEL_STATE_PENDING_RECONNECT",
                tableName: "Main",
                value: "Reconnecting",
                comment: ""
            )

        case .disconnected:
            return NSLocalizedString(
                "TUNNEL_STATE_DISCONNECTED",
                tableName: "Main",
                value: "Unsecured connection",
                comment: ""
            )

        case .waitingForConnectivity:
            return NSLocalizedString(
                "TUNNEL_STATE_WAITING_FOR_CONNECTIVITY",
                tableName: "Main",
                value: "Blocked connection",
                comment: ""
            )
        }
    }

    var localizedTitleForSelectLocationButton: String? {
        switch self {
        case .disconnecting(.reconnect), .pendingReconnect:
            return NSLocalizedString(
                "SWITCH_LOCATION_BUTTON_TITLE",
                tableName: "Main",
                value: "Select location",
                comment: ""
            )

        case .disconnected, .disconnecting(.nothing):
            return NSLocalizedString(
                "SELECT_LOCATION_BUTTON_TITLE",
                tableName: "Main",
                value: "Select location",
                comment: ""
            )
        case .connecting, .connected, .reconnecting, .waitingForConnectivity:
            return NSLocalizedString(
                "SWITCH_LOCATION_BUTTON_TITLE",
                tableName: "Main",
                value: "Switch location",
                comment: ""
            )
        }
    }

    var localizedAccessibilityLabel: String {
        switch self {
        case .connecting:
            return NSLocalizedString(
                "TUNNEL_STATE_CONNECTING_ACCESSIBILITY_LABEL",
                tableName: "Main",
                value: "Creating secure connection",
                comment: ""
            )

        case let .connected(tunnelInfo):
            return String(
                format: NSLocalizedString(
                    "TUNNEL_STATE_CONNECTED_ACCESSIBILITY_LABEL",
                    tableName: "Main",
                    value: "Secure connection. Connected to %@, %@",
                    comment: ""
                ),
                tunnelInfo.location.city,
                tunnelInfo.location.country
            )

        case .disconnected:
            return NSLocalizedString(
                "TUNNEL_STATE_DISCONNECTED_ACCESSIBILITY_LABEL",
                tableName: "Main",
                value: "Unsecured connection",
                comment: ""
            )

        case let .reconnecting(tunnelInfo):
            return String(
                format: NSLocalizedString(
                    "TUNNEL_STATE_RECONNECTING_ACCESSIBILITY_LABEL",
                    tableName: "Main",
                    value: "Reconnecting to %@, %@",
                    comment: ""
                ),
                tunnelInfo.location.city,
                tunnelInfo.location.country
            )

        case .waitingForConnectivity:
            return NSLocalizedString(
                "TUNNEL_STATE_WAITING_FOR_CONNECTIVITY_ACCESSIBILITY_LABEL",
                tableName: "Main",
                value: "Blocked connection",
                comment: ""
            )

        case .disconnecting(.nothing):
            return NSLocalizedString(
                "TUNNEL_STATE_DISCONNECTING_ACCESSIBILITY_LABEL",
                tableName: "Main",
                value: "Disconnecting",
                comment: ""
            )

        case .disconnecting(.reconnect), .pendingReconnect:
            return NSLocalizedString(
                "TUNNEL_STATE_PENDING_RECONNECT_ACCESSIBILITY_LABEL",
                tableName: "Main",
                value: "Reconnecting",
                comment: ""
            )
        }
    }

    func actionButtons(traitCollection: UITraitCollection) -> [TunnelControlActionButton] {
        switch (traitCollection.userInterfaceIdiom, traitCollection.horizontalSizeClass) {
        case (.phone, _), (.pad, .compact):
            switch self {
            case .disconnected, .disconnecting(.nothing):
                return [.selectLocation, .connect]

            case .connecting, .pendingReconnect, .disconnecting(.reconnect),
                 .waitingForConnectivity:
                return [.selectLocation, .cancel]

            case .connected, .reconnecting:
                return [.selectLocation, .disconnect]
            }

        case (.pad, .regular):
            switch self {
            case .disconnected, .disconnecting(.nothing):
                return [.connect]

            case .connecting, .pendingReconnect, .disconnecting(.reconnect),
                 .waitingForConnectivity:
                return [.cancel]

            case .connected, .reconnecting:
                return [.disconnect]
            }

        default:
            return []
        }
    }
}
