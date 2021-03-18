//
//  ConnectViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit
import NetworkExtension
import Logging

class ConnectViewController: UIViewController, RootContainment, TunnelObserver,
    SelectLocationDelegate
{
    private lazy var mainContentView: ConnectMainContentView = {
        let view = ConnectMainContentView(frame: UIScreen.main.bounds)
        view.translatesAutoresizingMaskIntoConstraints = false
        return view
    }()


    private let logger = Logger(label: "ConnectViewController")
    private let alertPresenter = AlertPresenter()

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

    private var showedAccountView = false

    override func viewDidLoad() {
        super.viewDidLoad()

        mainContentView.selectLocationButton.accessibilityIdentifier = "SelectLocationButton"
        mainContentView.splitDisconnectButton.primaryButton.accessibilityIdentifier = "DisconnectButton"

        mainContentView.connectionPanel.collapseButton.addTarget(self, action: #selector(handleConnectionPanelButton(_:)), for: .touchUpInside)
        mainContentView.connectButton.addTarget(self, action: #selector(handleConnect(_:)), for: .touchUpInside)
        mainContentView.splitDisconnectButton.primaryButton.addTarget(self, action: #selector(handleDisconnect(_:)), for: .touchUpInside)
        mainContentView.splitDisconnectButton.secondaryButton.addTarget(self, action: #selector(handleReconnect(_:)), for: .touchUpInside)

        mainContentView.selectLocationButton.addTarget(self, action: #selector(handleSelectLocation(_:)), for: .touchUpInside)

        view.addSubview(mainContentView)
        NSLayoutConstraint.activate([
            mainContentView.topAnchor.constraint(equalTo: view.topAnchor),
            mainContentView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            mainContentView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            mainContentView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])

        TunnelManager.shared.addObserver(self)
        self.tunnelState = TunnelManager.shared.tunnelState
    }

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)

        showAccountViewForExpiredAccount()
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

    // MARK: - SelectLocationDelegate

    func selectLocationViewController(_ controller: SelectLocationViewController, didSelectLocation location: RelayLocation) {
        controller.dismiss(animated: true) {
            let relayConstraints = RelayConstraints(location: .only(location))

            TunnelManager.shared.setRelayConstraints(relayConstraints) { [weak self] (result) in
                DispatchQueue.main.async {
                    guard let self = self else { return }

                    switch result {
                    case .success:
                        self.logger.debug("Updated relay constraints: \(relayConstraints)")
                        self.connectTunnel()

                    case .failure(let error):
                        self.logger.error(chainedError: error, message: "Failed to update relay constraints")
                    }
                }
            }
        }
    }

    func selectLocationViewControllerDidCancel(_ controller: SelectLocationViewController) {
        controller.dismiss(animated: true)
    }

    // MARK: - Private

    private func updateUserInterfaceForTunnelStateChange() {
        mainContentView.secureLabel.text = tunnelState.localizedTitleForSecureLabel.uppercased()
        mainContentView.secureLabel.textColor = tunnelState.textColorForSecureLabel

        mainContentView.connectButton.setTitle(tunnelState.localizedTitleForConnectButton, for: .normal)
        mainContentView.selectLocationButton.setTitle(tunnelState.localizedTitleForSelectLocationButton, for: .normal)
        mainContentView.splitDisconnectButton.primaryButton.setTitle(tunnelState.localizedTitleForDisconnectButton, for: .normal)
        mainContentView.setActionButtons(tunnelState.actionButtons)
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

    private func connectTunnel() {
        TunnelManager.shared.startTunnel { (result) in
            DispatchQueue.main.async {
                switch result {
                case .success:
                    break

                case .failure(let error):
                    self.logger.error(chainedError: error, message: "Failed to start the VPN tunnel")

                    let alertController = UIAlertController(
                        title: NSLocalizedString("Failed to start the VPN tunnel", comment: ""),
                        message: error.errorChainDescription,
                        preferredStyle: .alert
                    )
                    alertController.addAction(
                        UIAlertAction(title: NSLocalizedString("OK", comment: ""), style: .cancel)
                    )

                    self.alertPresenter.enqueue(alertController, presentingController: self)
                }
            }
        }
    }

    private func disconnectTunnel() {
        TunnelManager.shared.stopTunnel { (result) in
            if case .failure(let error) = result {
                self.logger.error(chainedError: error, message: "Failed to stop the VPN tunnel")

                let alertController = UIAlertController(
                    title: NSLocalizedString("Failed to stop the VPN tunnel", comment: ""),
                    message: error.errorChainDescription,
                    preferredStyle: .alert
                )
                alertController.addAction(
                    UIAlertAction(title: NSLocalizedString("OK", comment: ""), style: .cancel)
                )

                self.alertPresenter.enqueue(alertController, presentingController: self)
            }
        }
    }

    private func reconnectTunnel() {
        TunnelManager.shared.reconnectTunnel(completionHandler: nil)
    }

    private func showAccountViewForExpiredAccount() {
        guard !showedAccountView else { return }

        showedAccountView = true

        if let accountExpiry = Account.shared.expiry, AccountExpiry(date: accountExpiry).isExpired {
            rootContainerController?.showSettings(navigateTo: .account, animated: true)
        }
    }

    private func showSelectLocation() {
        let selectLocationController = SelectLocationNavigationController()
        selectLocationController.selectLocationDelegate = self

        // Disable root controller interaction
        rootContainerController?.view.isUserInteractionEnabled = false

        selectLocationController.prefetchData {
            self.present(selectLocationController, animated: true)

            // Re-enable root controller interaction
            self.rootContainerController?.view.isUserInteractionEnabled = true
        }
    }

    // MARK: - Actions

    @objc func handleConnectionPanelButton(_ sender: Any) {
        mainContentView.connectionPanel.toggleConnectionInfoVisibility()
    }

    @objc func handleConnect(_ sender: Any) {
        connectTunnel()
    }

    @objc func handleDisconnect(_ sender: Any) {
        disconnectTunnel()
    }

    @objc func handleReconnect(_ sender: Any) {
        reconnectTunnel()
    }

    @objc func handleSelectLocation(_ sender: Any) {
        showSelectLocation()
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

    var actionButtons: [ConnectMainContentView.ActionButton] {
        switch self {
        case .disconnected, .disconnecting:
            return [.selectLocation, .connect]

        case .connecting, .connected, .reconnecting:
            return [.selectLocation, .disconnect]
        }
    }

}
