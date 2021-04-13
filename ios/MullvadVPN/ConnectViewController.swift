//
//  ConnectViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit
import Logging

protocol ConnectViewControllerDelegate: class {
    func connectViewControllerShouldShowSelectLocationPicker(_ controller: ConnectViewController)
    func connectViewControllerShouldConnectTunnel(_ controller: ConnectViewController)
    func connectViewControllerShouldDisconnectTunnel(_ controller: ConnectViewController)
    func connectViewControllerShouldReconnectTunnel(_ controller: ConnectViewController)
}

class ConnectViewController: UIViewController, RootContainment, TunnelObserver
{
    weak var delegate: ConnectViewControllerDelegate?

    private let mainContentView: ConnectMainContentView = {
        let view = ConnectMainContentView(frame: UIScreen.main.bounds)
        view.translatesAutoresizingMaskIntoConstraints = false
        return view
    }()

    private let logger = Logger(label: "ConnectViewController")


    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    var preferredHeaderBarStyle: HeaderBarStyle {
        switch tunnelState {
        case .connecting, .reconnecting, .connected:
            return .secured

        case .disconnecting, .disconnected:
            return .unsecured
        }
    }

    var prefersHeaderBarHidden: Bool {
        return false
    }

    private var tunnelState: TunnelState = .disconnected {
        didSet {
            setNeedsHeaderBarStyleAppearanceUpdate()
            updateTunnelConnectionInfo()
            updateUserInterfaceForTunnelStateChange()
        }
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        mainContentView.connectionPanel.collapseButton.addTarget(self, action: #selector(handleConnectionPanelButton(_:)), for: .touchUpInside)
        mainContentView.connectButton.addTarget(self, action: #selector(handleConnect(_:)), for: .touchUpInside)
        mainContentView.splitDisconnectButton.primaryButton.addTarget(self, action: #selector(handleDisconnect(_:)), for: .touchUpInside)
        mainContentView.splitDisconnectButton.secondaryButton.addTarget(self, action: #selector(handleReconnect(_:)), for: .touchUpInside)

        mainContentView.selectLocationButton.addTarget(self, action: #selector(handleSelectLocation(_:)), for: .touchUpInside)

        TunnelManager.shared.addObserver(self)
        self.tunnelState = TunnelManager.shared.tunnelState

        addSubviews()
    }

    override func traitCollectionDidChange(_ previousTraitCollection: UITraitCollection?) {
        super.traitCollectionDidChange(previousTraitCollection)

        if previousTraitCollection?.userInterfaceIdiom != traitCollection.userInterfaceIdiom ||
            previousTraitCollection?.horizontalSizeClass != traitCollection.horizontalSizeClass {
            updateTraitDependentViews()
        }
    }

    func setMainContentHidden(_ isHidden: Bool, animated: Bool) {
        let actions = {
            self.mainContentView.containerView.alpha = isHidden ? 0 : 1
        }

        if animated {
            UIView.animate(withDuration: 0.25, animations: actions)
        } else {
            actions()
        }
    }

    private func addSubviews() {
        view.addSubview(mainContentView)
        NSLayoutConstraint.activate([
            mainContentView.topAnchor.constraint(equalTo: view.topAnchor),
            mainContentView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            mainContentView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            mainContentView.bottomAnchor.constraint(equalTo: view.bottomAnchor)
        ])
    }

    // MARK: - TunnelObserver

    func tunnelStateDidChange(tunnelState: TunnelState) {
        DispatchQueue.main.async {
            self.tunnelState = tunnelState
        }
    }

    func tunnelPublicKeyDidChange(publicKeyWithMetadata: PublicKeyWithMetadata?) {
        // no-op
    }

    // MARK: - Private

    private func updateUserInterfaceForTunnelStateChange() {
        mainContentView.secureLabel.text = tunnelState.localizedTitleForSecureLabel.uppercased()
        mainContentView.secureLabel.textColor = tunnelState.textColorForSecureLabel

        mainContentView.connectButton.setTitle(tunnelState.localizedTitleForConnectButton, for: .normal)
        mainContentView.selectLocationButton.setTitle(tunnelState.localizedTitleForSelectLocationButton, for: .normal)
        mainContentView.splitDisconnectButton.primaryButton.setTitle(tunnelState.localizedTitleForDisconnectButton, for: .normal)

        updateTraitDependentViews()
    }

    private func updateTraitDependentViews() {
        mainContentView.setActionButtons(tunnelState.actionButtons(traitCollection: self.traitCollection))
    }

    private func attributedStringForLocation(string: String) -> NSAttributedString {
        let paragraphStyle = NSMutableParagraphStyle()
        paragraphStyle.lineSpacing = 0
        paragraphStyle.lineHeightMultiple = 0.80
        return NSAttributedString(string: string, attributes: [
            .paragraphStyle: paragraphStyle])
    }

    private func updateTunnelConnectionInfo() {
        switch tunnelState {
        case .connected(let connectionInfo),
             .reconnecting(let connectionInfo):
            mainContentView.cityLabel.attributedText = attributedStringForLocation(string: connectionInfo.location.city)
            mainContentView.countryLabel.attributedText = attributedStringForLocation(string: connectionInfo.location.country)

            mainContentView.connectionPanel.dataSource = ConnectionPanelData(
                inAddress: "\(connectionInfo.ipv4Relay) UDP",
                outAddress: nil
            )
            mainContentView.connectionPanel.isHidden = false
            mainContentView.connectionPanel.collapseButton.setTitle(connectionInfo.hostname, for: .normal)

        case .connecting, .disconnected, .disconnecting:
            mainContentView.cityLabel.attributedText = attributedStringForLocation(string: " ")
            mainContentView.countryLabel.attributedText = attributedStringForLocation(string: " ")
            mainContentView.connectionPanel.dataSource = nil
            mainContentView.connectionPanel.isHidden = true
        }
    }

    // MARK: - Actions

    @objc func handleConnectionPanelButton(_ sender: Any) {
        mainContentView.connectionPanel.toggleConnectionInfoVisibility()
    }

    @objc func handleConnect(_ sender: Any) {
        delegate?.connectViewControllerShouldConnectTunnel(self)
    }

    @objc func handleDisconnect(_ sender: Any) {
        delegate?.connectViewControllerShouldDisconnectTunnel(self)
    }

    @objc func handleReconnect(_ sender: Any) {
        delegate?.connectViewControllerShouldReconnectTunnel(self)
    }

    @objc func handleSelectLocation(_ sender: Any) {
        delegate?.connectViewControllerShouldShowSelectLocationPicker(self)
    }
}

private extension TunnelState {

    var textColorForSecureLabel: UIColor {
        switch self {
        case .connecting, .reconnecting:
            return .white

        case .connected:
            return .successColor

        case .disconnecting, .disconnected:
            return .dangerColor
        }
    }

    var localizedTitleForSecureLabel: String {
        switch self {
        case .connecting, .reconnecting:
            return NSLocalizedString("Creating secure connection", comment: "")

        case .connected:
            return NSLocalizedString("Secure connection", comment: "")

        case .disconnecting, .disconnected:
            return NSLocalizedString("Unsecured connection", comment: "")
        }
    }

    var localizedTitleForSelectLocationButton: String? {
        switch self {
        case .disconnected, .disconnecting:
            return NSLocalizedString("Select location", comment: "")
        case .connecting, .connected, .reconnecting:
            return NSLocalizedString("Switch location", comment: "")
        }
    }

    var localizedTitleForConnectButton: String? {
        return NSLocalizedString("Secure connection", comment: "")
    }

    var localizedTitleForDisconnectButton: String? {
        switch self {
        case .connecting:
            return NSLocalizedString("Cancel", comment: "")
        case .connected, .reconnecting:
            return NSLocalizedString("Disconnect", comment: "")
        case .disconnecting, .disconnected:
            return nil
        }
    }

    func actionButtons(traitCollection: UITraitCollection) -> [ConnectMainContentView.ActionButton] {
        switch (traitCollection.userInterfaceIdiom, traitCollection.horizontalSizeClass) {
        case (.phone, _), (.pad, .compact):
            switch self {
            case .disconnected, .disconnecting:
                return [.selectLocation, .connect]

            case .connecting, .connected, .reconnecting:
                return [.selectLocation, .disconnect]
            }

        case (.pad, .regular):
            switch self {
            case .disconnected, .disconnecting:
                return [.connect]

            case .connecting, .connected, .reconnecting:
                return [.disconnect]
            }

        default:
            fatalError("Not supported")
        }
    }

}
