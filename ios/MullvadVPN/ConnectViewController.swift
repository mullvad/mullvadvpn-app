//
//  ConnectViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Combine
import UIKit
import NetworkExtension
import os

class ConnectViewController: UIViewController, RootContainment, TunnelControlViewControllerDelegate {

    @IBOutlet var secureLabel: UILabel!
    @IBOutlet var countryLabel: UILabel!

    private var setRelaysSubscriber: AnyCancellable?
    private var startStopTunnelSubscriber: AnyCancellable?
    private var tunnelStateSubscriber: AnyCancellable?

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

    private var tunnelState: TunnelState = .disconnected {
        didSet {
            setNeedsHeaderBarStyleAppearanceUpdate()
            updateSecureLabel()
            updateTunnelConnectionInfo()
        }
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        tunnelStateSubscriber = TunnelManager.shared.$tunnelState
            .receive(on: DispatchQueue.main)
            .assign(to: \.tunnelState, on: self)
    }

    override func prepare(for segue: UIStoryboardSegue, sender: Any?) {
        if case .embedTunnelControls = SegueIdentifier.Connect.from(segue: segue) {
            let tunnelControlController = segue.destination as! TunnelControlViewController
            tunnelControlController.view.translatesAutoresizingMaskIntoConstraints = false
            tunnelControlController.delegate = self
        }
    }

    // MARK: - TunnelControlViewControllerDelegate

    func tunnelControlViewController(_ controller: TunnelControlViewController, handleAction action: TunnelControlAction) {
        switch action {
        case .connect:
            connectTunnel()

        case .disconnect:
            disconnectTunnel()

        case .selectLocation:
            performSegue(withIdentifier: SegueIdentifier.Connect.showRelaySelector.rawValue, sender: self)
        }
    }

    // MARK: - Private

    private func updateSecureLabel() {
        secureLabel.text = tunnelState.textForSecureLabel()
        secureLabel.textColor = tunnelState.textColorForSecureLabel()
    }

    private func updateTunnelConnectionInfo() {
        switch tunnelState {
        case .connected(let connectionInfo),
             .reconnecting(let connectionInfo):
            countryLabel.text = "\(connectionInfo.hostname)\nIn: \(connectionInfo.ipv4Relay)"

        case .connecting, .disconnected, .disconnecting:
            countryLabel.text = ""
        }
    }

    private func connectTunnel() {
        startStopTunnelSubscriber = TunnelManager.shared.startTunnel()
            .receive(on: DispatchQueue.main)
            .sink(receiveCompletion: { (completion) in
                if case .failure(let error) = completion {
                    os_log(.error, "Failed to start the tunnel: %{public}s", error.localizedDescription)
                }
            })
    }

    private func disconnectTunnel() {
        startStopTunnelSubscriber = TunnelManager.shared.stopTunnel()
            .receive(on: DispatchQueue.main)
            .sink(receiveCompletion: { (_) in
                // no-op
            })
    }

    // MARK: - Actions

    @IBAction func unwindFromSelectLocation(segue: UIStoryboardSegue) {
        guard let selectLocationController = segue.source as? SelectLocationController else { return }
        guard let selectedItem = selectLocationController.selectedItem else { return }

        let relayConstraints = RelayConstraints(location: .only(selectedItem.relayLocation))

        setRelaysSubscriber = TunnelManager.shared.setRelayConstraints(relayConstraints)
            .receive(on: DispatchQueue.main)
            .sink(receiveCompletion: { (completion) in
                switch completion {
                case .finished:
                    os_log(.debug, "Updated relay constraints: %{public}s", String(reflecting: relayConstraints))
                    self.connectTunnel()

                case .failure(let error):
                    os_log(.error, "Failed to update relay constraints: %{public}s", error.localizedDescription)
                }
            })
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
