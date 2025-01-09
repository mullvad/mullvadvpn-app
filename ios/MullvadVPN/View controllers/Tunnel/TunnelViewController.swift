//
//  TunnelViewController.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-10.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Combine
import MapKit
import MullvadLogging
import MullvadSettings
import MullvadTypes
import SwiftUI

class TunnelViewController: UIViewController, RootContainment {
    private let logger = Logger(label: "TunnelViewController")
    private let interactor: TunnelViewControllerInteractor
    private var tunnelState: TunnelState = .disconnected
    private var connectionViewViewModel: ConnectionViewViewModel
    private var indicatorsViewViewModel: FeatureIndicatorsViewModel
    private var connectionView: ConnectionView
    private var connectionController: UIHostingController<ConnectionView>?

    var shouldShowSelectLocationPicker: (() -> Void)?
    var shouldShowCancelTunnelAlert: (() -> Void)?

    private let mapViewController = MapViewController()

    override var preferredStatusBarStyle: UIStatusBarStyle {
        .lightContent
    }

    var preferredHeaderBarPresentation: HeaderBarPresentation {
        switch interactor.deviceState {
        case .loggedIn, .revoked:
            return HeaderBarPresentation(
                style: tunnelState.isSecured ? .secured : .unsecured,
                showsDivider: false
            )
        case .loggedOut:
            return HeaderBarPresentation(style: .default, showsDivider: true)
        }
    }

    var prefersHeaderBarHidden: Bool {
        false
    }

    init(interactor: TunnelViewControllerInteractor) {
        self.interactor = interactor

        tunnelState = interactor.tunnelStatus.state
        connectionViewViewModel = ConnectionViewViewModel(tunnelStatus: interactor.tunnelStatus)
        indicatorsViewViewModel = FeatureIndicatorsViewModel(
            tunnelSettings: interactor.tunnelSettings,
            ipOverrides: interactor.ipOverrides
        )

        connectionView = ConnectionView(
            connectionViewModel: self.connectionViewViewModel,
            indicatorsViewModel: self.indicatorsViewViewModel
        )

        super.init(nibName: nil, bundle: nil)

        // When content size is updated in SwiftUI we need to explicitly tell UIKit to
        // update its view size. This is not necessary on iOS 16 where we can set
        // hostingController.sizingOptions instead.
        connectionView.onContentUpdate = { [weak self] in
            self?.connectionController?.view.setNeedsUpdateConstraints()
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        interactor.didUpdateDeviceState = { [weak self] _, _ in
            self?.setNeedsHeaderBarStyleAppearanceUpdate()
        }

        interactor.didUpdateTunnelStatus = { [weak self] tunnelStatus in
            self?.connectionViewViewModel.tunnelStatus = tunnelStatus
            self?.setTunnelState(tunnelStatus.state, animated: true)
            self?.view.setNeedsLayout()
        }

        interactor.didGetOutgoingAddress = { [weak self] outgoingConnectionInfo in
            self?.connectionViewViewModel.outgoingConnectionInfo = outgoingConnectionInfo
        }

        interactor.didUpdateTunnelSettings = { [weak self] tunnelSettings in
            self?.indicatorsViewViewModel.tunnelSettings = tunnelSettings
        }

        interactor.didUpdateIpOverrides = { [weak self] overrides in
            self?.indicatorsViewViewModel.ipOverrides = overrides
        }

        connectionView.action = { [weak self] action in
            switch action {
            case .connect:
                self?.interactor.startTunnel()

            case .cancel:
                if case .waitingForConnectivity(.noConnection) = self?.interactor.tunnelStatus.state {
                    self?.shouldShowCancelTunnelAlert?()
                } else {
                    self?.interactor.stopTunnel()
                }

            case .disconnect:
                self?.interactor.stopTunnel()

            case .reconnect:
                self?.interactor.reconnectTunnel(selectNewRelay: true)

            case .selectLocation:
                self?.shouldShowSelectLocationPicker?()
            }
        }

        addMapController()
        addContentView()
        updateMap(animated: false)
    }

    func setMainContentHidden(_ isHidden: Bool, animated: Bool) {
        let actions = {
            _ = self.connectionView.opacity(isHidden ? 0 : 1)
        }

        if animated {
            UIView.animate(withDuration: 0.25, animations: actions)
        } else {
            actions()
        }
    }

    // MARK: - Private

    private func setTunnelState(_ tunnelState: TunnelState, animated: Bool) {
        self.tunnelState = tunnelState

        setNeedsHeaderBarStyleAppearanceUpdate()

        guard isViewLoaded else { return }

        updateMap(animated: animated)
    }

    private func updateMap(animated: Bool) {
        switch tunnelState {
        case let .connecting(tunnelRelays, _, _):
            mapViewController.removeLocationMarker()
            mapViewController.setCenter(tunnelRelays?.exit.location.geoCoordinate, animated: animated)
            connectionViewViewModel.showsActivityIndicator = true

        case let .reconnecting(tunnelRelays, _, _), let .negotiatingEphemeralPeer(tunnelRelays, _, _, _):
            mapViewController.removeLocationMarker()
            mapViewController.setCenter(tunnelRelays.exit.location.geoCoordinate, animated: animated)
            connectionViewViewModel.showsActivityIndicator = true

        case let .connected(tunnelRelays, _, _):
            let center = tunnelRelays.exit.location.geoCoordinate
            mapViewController.setCenter(center, animated: animated) {
                self.connectionViewViewModel.showsActivityIndicator = false

                // Connection can change during animation, so make sure we're still connected before adding marker.
                if case .connected = self.tunnelState {
                    self.mapViewController.addLocationMarker(coordinate: center)
                }
            }

        case .pendingReconnect:
            mapViewController.removeLocationMarker()
            connectionViewViewModel.showsActivityIndicator = true

        case .waitingForConnectivity, .error:
            mapViewController.removeLocationMarker()
            connectionViewViewModel.showsActivityIndicator = false

        case .disconnected, .disconnecting:
            mapViewController.removeLocationMarker()
            mapViewController.setCenter(nil, animated: animated)
            connectionViewViewModel.showsActivityIndicator = false
        }
    }

    private func addMapController() {
        let mapView = mapViewController.view!

        addChild(mapViewController)
        mapViewController.didMove(toParent: self)

        view.addConstrainedSubviews([mapView]) {
            mapView.pinEdgesToSuperview()
        }
    }

    private func addContentView() {
        let connectionController = UIHostingController(rootView: connectionView)
        self.connectionController = connectionController

        let connectionViewProxy = connectionController.view!
        connectionViewProxy.backgroundColor = .clear

        addChild(connectionController)
        connectionController.didMove(toParent: self)

        view.addConstrainedSubviews([connectionViewProxy]) {
            connectionViewProxy.pinEdgesToSuperview(.all())
        }
    }
}
