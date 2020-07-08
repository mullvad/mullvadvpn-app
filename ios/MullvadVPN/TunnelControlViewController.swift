//
//  TunnelControlView.swift
//  MullvadVPN
//
//  Created by pronebird on 01/11/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

enum TunnelControlAction {
    /// An action emitted only when the tunnel is down
    case connect

    /// An action emitted when user either selects to cancel the connection or disconnect when
    /// the tunnel is already connected
    case disconnect

    /// An action emitted when user requests to either select the location when the tunnel is down
    /// or change the location when the tunnel is connecting or connected.
    case selectLocation
}

protocol TunnelControlViewControllerDelegate: class {
    func tunnelControlViewController(_ controller: TunnelControlViewController, handleAction action: TunnelControlAction) -> Void
}

class TunnelControlViewController: UIViewController, TunnelObserver {

    @IBOutlet var disconnectedView: UIView!
    @IBOutlet var connectingView: UIView!
    @IBOutlet var connectedView: UIView!

    weak var delegate: TunnelControlViewControllerDelegate?

    private var controlsView: UIView?

    override func viewDidLoad() {
        super.viewDidLoad()

        TunnelManager.shared.addObserver(self)
        self.didReceiveTunnelState(TunnelManager.shared.tunnelState)
    }

    // MARK: - TunnelObserver

    func tunnelStateDidChange(tunnelState: TunnelState) {
        DispatchQueue.main.async {
            self.didReceiveTunnelState(tunnelState)
        }
    }

    func tunnelPublicKeyDidChange(publicKey: WireguardPublicKey?) {
        // no-op
    }

    /// MARK: - Private

    private func didReceiveTunnelState(_ tunnelState: TunnelState) {
        switch tunnelState {
        case .disconnected:
            addControlsView(disconnectedView)

        case .connecting:
            addControlsView(connectingView)

        case .connected, .reconnecting, .disconnecting:
            addControlsView(connectedView)
        }
    }

    private func addControlsView(_ nextControlsView: UIView) {
        guard controlsView != nextControlsView else { return }

        controlsView?.removeFromSuperview()
        controlsView = nextControlsView

        nextControlsView.translatesAutoresizingMaskIntoConstraints = false

        view.addSubview(nextControlsView)

        NSLayoutConstraint.activate([
            nextControlsView.topAnchor.constraint(equalTo: view.topAnchor),
            nextControlsView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
            nextControlsView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            nextControlsView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
        ])
    }

    // MARK: - Actions

    @IBAction func handleSecureConnection(_ sender: Any) {
        delegate?.tunnelControlViewController(self, handleAction: .connect)
    }

    @IBAction func handleDisconnect(_ sender: Any) {
        delegate?.tunnelControlViewController(self, handleAction: .disconnect)
    }

    @IBAction func handleSelectLocation(_ sender: Any) {
        delegate?.tunnelControlViewController(self, handleAction: .selectLocation)
    }

}
