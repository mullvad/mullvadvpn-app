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
    @IBOutlet var secureLabel: UILabel!
    @IBOutlet var countryLabel: UILabel!
    @IBOutlet var cityLabel: UILabel!
    @IBOutlet var connectionPanel: ConnectionPanelView!
    @IBOutlet var buttonsStackView: UIStackView!

    private let logger = Logger(label: "ConnectViewController")

    private let connectButton = AppButton(style: .success)
    private let selectLocationButton = AppButton(style: .translucentNeutral)
    private let splitDisconnectButtonView = DisconnectSplitButton()

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
            updateSecureLabel()
            updateTunnelConnectionInfo()
            updateButtons()
        }
    }

    private var showedAccountView = false

    override func viewDidLoad() {
        super.viewDidLoad()

        for button in [connectButton, selectLocationButton] {
            button.titleLabel?.font = UIFont.systemFont(ofSize: 18, weight: .semibold)
        }

        connectionPanel.collapseButton.addTarget(self, action: #selector(handleConnectionPanelButton(_:)), for: .touchUpInside)
        connectButton.addTarget(self, action: #selector(handleConnect(_:)), for: .touchUpInside)
        splitDisconnectButtonView.primaryButton.addTarget(self, action: #selector(handleDisconnect(_:)), for: .touchUpInside)
        splitDisconnectButtonView.secondaryButton.addTarget(self, action: #selector(handleReconnect(_:)), for: .touchUpInside)

        selectLocationButton.addTarget(self, action: #selector(handleSelectLocation(_:)), for: .touchUpInside)

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

    func tunnelPublicKeyDidChange(publicKey: WireguardPublicKey?) {
        // no-op
    }

    // MARK: - SelectLocationDelegate

    func selectLocationController(_ controller: SelectLocationController, didSelectLocation location: RelayLocation) {
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

    func selectLocationControllerDidCancel(_ controller: SelectLocationController) {
        controller.dismiss(animated: true)
    }

    // MARK: - Private

    private func updateButtons() {
        switch tunnelState {
        case .disconnected, .disconnecting:
            selectLocationButton.setTitle(NSLocalizedString("Select location", comment: ""), for: .normal)
            connectButton.setTitle(NSLocalizedString("Secure connection", comment: ""), for: .normal)

            setArrangedButtons([selectLocationButton, connectButton])

        case .connecting:
            selectLocationButton.setTitle(NSLocalizedString("Switch location", comment: ""), for: .normal)
            splitDisconnectButtonView.primaryButton.setTitle(NSLocalizedString("Cancel", comment: ""), for: .normal)

            setArrangedButtons([selectLocationButton, splitDisconnectButtonView])

        case .connected, .reconnecting:
            selectLocationButton.setTitle(NSLocalizedString("Switch location", comment: ""), for: .normal)
            splitDisconnectButtonView.primaryButton.setTitle(NSLocalizedString("Disconnect", comment: ""), for: .normal)

            setArrangedButtons([selectLocationButton, splitDisconnectButtonView])
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

    private func updateSecureLabel() {
        secureLabel.text = tunnelState.textForSecureLabel().uppercased()
        secureLabel.textColor = tunnelState.textColorForSecureLabel()
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
            cityLabel.attributedText = attributedStringForLocation(string: connectionInfo.location.city)
            countryLabel.attributedText = attributedStringForLocation(string: connectionInfo.location.country)

            connectionPanel.dataSource = ConnectionPanelData(
                inAddress: "\(connectionInfo.ipv4Relay) UDP",
                outAddress: nil
            )
            connectionPanel.isHidden = false
            connectionPanel.collapseButton.setTitle(connectionInfo.hostname, for: .normal)

        case .connecting, .disconnected, .disconnecting:
            cityLabel.attributedText = attributedStringForLocation(string: " ")
            countryLabel.attributedText = attributedStringForLocation(string: " ")
            connectionPanel.dataSource = nil
            connectionPanel.isHidden = true
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
        connectionPanel.toggleConnectionInfoVisibility()
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

    func textColorForSecureLabel() -> UIColor {
        switch self {
        case .connecting, .reconnecting:
            return .white

        case .connected:
            return .successColor

        case .disconnecting, .disconnected:
            return .dangerColor
        }
    }

    func textForSecureLabel() -> String {
        switch self {
        case .connecting, .reconnecting:
            return NSLocalizedString("Creating secure connection", comment: "")

        case .connected:
            return NSLocalizedString("Secure connection", comment: "")

        case .disconnecting, .disconnected:
            return NSLocalizedString("Unsecured connection", comment: "")
        }
    }

}
