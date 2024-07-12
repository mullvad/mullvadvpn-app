//
//  TunnelControlView.swift
//  MullvadVPN
//
//  Created by pronebird on 09/03/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import MapKit
import MullvadREST
import MullvadTypes
import PacketTunnelCore
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
    private let secureLabel = makeBoldTextLabel(ofSize: 20, numberOfLines: 0)
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
        button.accessibilityIdentifier = .secureConnectionButton
        button.translatesAutoresizingMaskIntoConstraints = false
        return button
    }()

    private let cancelButton: AppButton = {
        let button = AppButton(style: .translucentDanger)
        button.accessibilityIdentifier = .cancelButton
        button.translatesAutoresizingMaskIntoConstraints = false
        return button
    }()

    private let selectLocationButton: AppButton = {
        let button = AppButton(style: .translucentNeutral)
        button.accessibilityIdentifier = .selectLocationButton
        button.translatesAutoresizingMaskIntoConstraints = false
        return button
    }()

    private let selectLocationButtonBlurView: TranslucentButtonBlurView
    private let connectButtonBlurView: TranslucentButtonBlurView
    private let cancelButtonBlurView: TranslucentButtonBlurView

    private let splitDisconnectButton: DisconnectSplitButton = {
        let button = DisconnectSplitButton()
        button.primaryButton.accessibilityIdentifier = .disconnectButton
        button.translatesAutoresizingMaskIntoConstraints = false
        return button
    }()

    private let containerView: UIView = {
        let view = UIView()
        view.translatesAutoresizingMaskIntoConstraints = false
        return view
    }()

    private var traitConstraints = [NSLayoutConstraint]()
    private var viewModel: TunnelControlViewModel?

    var actionHandler: ((TunnelControlAction) -> Void)?

    var mapCenterAlignmentView: UIView {
        activityIndicator
    }

    override init(frame: CGRect) {
        selectLocationButtonBlurView = TranslucentButtonBlurView(button: selectLocationButton)
        connectButtonBlurView = TranslucentButtonBlurView(button: connectButton)
        cancelButtonBlurView = TranslucentButtonBlurView(button: cancelButton)

        super.init(frame: frame)

        backgroundColor = .clear
        directionalLayoutMargins = UIMetrics.contentLayoutMargins
        accessibilityContainerType = .semanticGroup
        accessibilityIdentifier = .tunnelControlView

        addSubviews()
        addButtonHandlers()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func update(with model: TunnelControlViewModel) {
        viewModel = model
        let tunnelState = model.tunnelStatus.state
        secureLabel.text = model.secureLabelText
        secureLabel.textColor = tunnelState.textColorForSecureLabel
        selectLocationButtonBlurView.isEnabled = model.enableButtons
        connectButtonBlurView.isEnabled = model.enableButtons
        cityLabel.attributedText = attributedStringForLocation(string: model.city)
        countryLabel.attributedText = attributedStringForLocation(string: model.country)
        connectionPanel.connectedRelayName = model.connectedRelayName
        connectionPanel.dataSource = model.connectionPanel

        updateSecureLabel(tunnelState: tunnelState)
        updateActionButtons(tunnelState: tunnelState)
        if tunnelState.isSecured {
            updateTunnelRelays(tunnelRelays: tunnelState.relays)
        } else {
            updateTunnelRelays(tunnelRelays: nil)
        }
    }

    func setAnimatingActivity(_ isAnimating: Bool) {
        if isAnimating {
            activityIndicator.startAnimating()
        } else {
            activityIndicator.stopAnimating()
        }
    }

    private func updateActionButtons(tunnelState: TunnelState) {
        let actionButtons = tunnelState.actionButtons(traitCollection: traitCollection)
        let views = actionButtons.map { self.view(forActionButton: $0) }

        updateButtonTitles(tunnelState: tunnelState)
        updateButtonEnabledStates(shouldEnableButtons: tunnelState.shouldEnableButtons)
        setArrangedButtons(views)
    }

    private func updateSecureLabel(tunnelState: TunnelState) {
        secureLabel.text = tunnelState.localizedTitleForSecureLabel.uppercased()
        secureLabel.textColor = tunnelState.textColorForSecureLabel

        switch tunnelState {
        case .connected:
            secureLabel.accessibilityIdentifier = .connectionStatusConnectedLabel
        case .connecting:
            secureLabel.accessibilityIdentifier = .connectionStatusConnectingLabel
        default:
            secureLabel.accessibilityIdentifier = .connectionStatusNotConnectedLabel
        }
    }

    private func updateButtonTitles(tunnelState: TunnelState) {
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
                value: tunnelState == .waitingForConnectivity(.noConnection) ? "Disconnect" : "Cancel",
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

    private func updateButtonEnabledStates(shouldEnableButtons: Bool) {
        selectLocationButtonBlurView.isEnabled = shouldEnableButtons
        connectButtonBlurView.isEnabled = shouldEnableButtons
    }

    private func updateTunnelRelays(tunnelRelays: SelectedRelays?) {
        if let tunnelRelays {
            cityLabel.attributedText = attributedStringForLocation(
                string: tunnelRelays.exit.location.city // TODO: Multihop
            )
            countryLabel.attributedText = attributedStringForLocation(
                string: tunnelRelays.exit.location.country // TODO: Multihop
            )

            connectionPanel.isHidden = false
            connectionPanel.connectedRelayName = tunnelRelays.exit.hostname // TODO: Multihop
        } else {
            countryLabel.attributedText = attributedStringForLocation(string: " ")
            cityLabel.attributedText = attributedStringForLocation(string: " ")
            connectionPanel.dataSource = nil
            connectionPanel.isHidden = true
        }

        locationContainerView.accessibilityLabel = viewModel?.tunnelStatus.state.localizedAccessibilityLabel
    }

    // MARK: - Private

    private func addSubviews() {
        for subview in [secureLabel, countryLabel, cityLabel] {
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

            locationContainerView.topAnchor.constraint(greaterThanOrEqualTo: containerView.topAnchor),
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

            countryLabel.topAnchor.constraint(equalTo: secureLabel.bottomAnchor, constant: 8),
            countryLabel.leadingAnchor.constraint(equalTo: locationContainerView.leadingAnchor),
            countryLabel.trailingAnchor.constraint(equalTo: locationContainerView.trailingAnchor),

            cityLabel.topAnchor.constraint(equalTo: countryLabel.bottomAnchor, constant: 8),
            cityLabel.leadingAnchor.constraint(equalTo: locationContainerView.leadingAnchor),
            cityLabel.trailingAnchor.constraint(equalTo: locationContainerView.trailingAnchor),
            cityLabel.bottomAnchor.constraint(equalTo: locationContainerView.bottomAnchor),

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
            action: #selector(handleCancel),
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

        layoutConstraints.append(
            containerView.trailingAnchor.constraint(equalTo: layoutMarginsGuide.trailingAnchor)
        )

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
            return selectLocationButtonBlurView
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

    private class func makeBoldTextLabel(ofSize fontSize: CGFloat, numberOfLines: Int = 1) -> UILabel {
        let textLabel = UILabel()
        textLabel.translatesAutoresizingMaskIntoConstraints = false
        textLabel.font = UIFont.boldSystemFont(ofSize: fontSize)
        textLabel.textColor = .white
        textLabel.numberOfLines = numberOfLines
        return textLabel
    }

    // MARK: - Actions

    @objc private func handleConnect() {
        actionHandler?(.connect)
    }

    @objc private func handleCancel() {
        actionHandler?(.cancel)
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
    func actionButtons(traitCollection: UITraitCollection) -> [TunnelControlActionButton] {
        switch self {
        case .disconnected, .disconnecting(.nothing), .waitingForConnectivity(.noNetwork):
            [.selectLocation, .connect]

        case .connecting, .pendingReconnect, .disconnecting(.reconnect),
             .waitingForConnectivity(.noConnection):
            [.selectLocation, .cancel]

        case .negotiatingPostQuantumKey:
            [.selectLocation, .cancel]

        case .connected, .reconnecting, .error:
            [.selectLocation, .disconnect]
        }
    }

    // swiftlint:disable:next file_length
}
