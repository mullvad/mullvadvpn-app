//
//  TunnelViewController.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-10.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import MapKit
import MullvadLogging
import MullvadREST
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
    private var searchController: SearchAnythingViewController?
    private let relays: [RelayWithLocation<REST.ServerRelay>]

    var shouldShowSelectLocationPicker: (() -> Void)?
    var shouldShowCancelTunnelAlert: (() -> Void)?
    var didSelect: ((SearchAnythingViewController.Item) -> Void)?

    let activityIndicator: SpinnerActivityIndicatorView = {
        let activityIndicator = SpinnerActivityIndicatorView(style: .large)
        activityIndicator.translatesAutoresizingMaskIntoConstraints = false
        activityIndicator.tintColor = .white
        activityIndicator.setContentHuggingPriority(.defaultHigh, for: .horizontal)
        activityIndicator.setContentCompressionResistancePriority(.defaultHigh, for: .horizontal)
        return activityIndicator
    }()

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

    var prefersNotificationBarHidden: Bool {
        false
    }

    init(interactor: TunnelViewControllerInteractor, relays: [RelayWithLocation<REST.ServerRelay>]) {
        self.interactor = interactor
        self.relays = relays

        tunnelState = interactor.tunnelStatus.state
        connectionViewViewModel = ConnectionViewViewModel(
            tunnelStatus: interactor.tunnelStatus,
            relayConstraints: interactor.tunnelSettings.relayConstraints,
            relayCache: RelayCache(cacheDirectory: ApplicationConfiguration.containerURL),
            customListRepository: CustomListRepository()
        )
        indicatorsViewViewModel = FeatureIndicatorsViewModel(
            tunnelSettings: interactor.tunnelSettings,
            ipOverrides: interactor.ipOverrides,
            tunnelState: tunnelState
        )

        connectionView = ConnectionView(
            connectionViewModel: connectionViewViewModel,
            indicatorsViewModel: indicatorsViewViewModel
        )

        super.init(nibName: nil, bundle: nil)
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
            self?.connectionViewViewModel.update(tunnelStatus: tunnelStatus)
            self?.setTunnelState(tunnelStatus.state, animated: true)
            self?.indicatorsViewViewModel.tunnelState = tunnelStatus.state
            self?.view.setNeedsLayout()
        }

        interactor.didGetOutgoingAddress = { [weak self] outgoingConnectionInfo in
            self?.connectionViewViewModel.outgoingConnectionInfo = outgoingConnectionInfo
        }

        interactor.didUpdateTunnelSettings = { [weak self] tunnelSettings in
            self?.indicatorsViewViewModel.tunnelSettings = tunnelSettings
            self?.connectionViewViewModel.relayConstraints = tunnelSettings.relayConstraints
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
        addActivityIndicator()
        addConnectionView()
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

    func toggleSearchController() {
        guard searchController == nil else {
            hideSearchController()
            return
        }

        let controller = SearchAnythingViewController(relays: relays)
        searchController = controller

        addChild(controller)
        controller.didMove(toParent: self)

        controller.didSelect = { [weak self] item in
            self?.toggleSearchController()
            self?.didSelect?(item)
        }

        view.addConstrainedSubviews([controller.view]) {
            controller.view.pinEdgesToSuperview(.all().excluding(.top))
            controller.view.topAnchor.constraint(equalTo: view.topAnchor, constant: 148)
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
            activityIndicator.startAnimating()

        case let .reconnecting(tunnelRelays, _, _), let .negotiatingEphemeralPeer(tunnelRelays, _, _, _):
            activityIndicator.startAnimating()
            mapViewController.removeLocationMarker()
            mapViewController.setCenter(tunnelRelays.exit.location.geoCoordinate, animated: animated)

        case let .connected(tunnelRelays, _, _):
            let center = tunnelRelays.exit.location.geoCoordinate
            mapViewController.setCenter(center, animated: animated)
            activityIndicator.stopAnimating()
            mapViewController.addLocationMarker(coordinate: center)

        case .pendingReconnect:
            activityIndicator.startAnimating()
            mapViewController.removeLocationMarker()

        case .waitingForConnectivity, .error:
            activityIndicator.stopAnimating()
            mapViewController.removeLocationMarker()

        case .disconnected, .disconnecting:
            activityIndicator.stopAnimating()
            mapViewController.removeLocationMarker()
            mapViewController.setCenter(nil, animated: animated)
        }
    }

    private func addMapController() {
        let mapView = mapViewController.view!

        addChild(mapViewController)
        mapViewController.alignmentView = activityIndicator
        mapViewController.didMove(toParent: self)

        view.addConstrainedSubviews([mapView]) {
            mapView.pinEdgesToSuperview()
        }
    }

    /// Computes a constraint multiplier based on the screen size
    private func computeHeightBreakpointMultiplier() -> CGFloat {
        let screenBounds = UIWindow().screen.coordinateSpace.bounds
        return screenBounds.height < 700 ? 2.0 : 1.5
    }

    private func addActivityIndicator() {
        // If the device doesn't have a lot of vertical screen estate, center the progress view higher on the map
        // so the connection view details do not shadow it unless fully expanded if possible
        let heightConstraintMultiplier = computeHeightBreakpointMultiplier()

        let verticalCenteredAnchor = activityIndicator.centerYAnchor.anchorWithOffset(to: view.centerYAnchor)
        view.addConstrainedSubviews([activityIndicator]) {
            activityIndicator.centerXAnchor.constraint(equalTo: view.centerXAnchor)
            verticalCenteredAnchor.constraint(
                equalTo: activityIndicator.heightAnchor,
                multiplier: heightConstraintMultiplier
            )
        }
    }

    private func addConnectionView() {
        let connectionController = UIHostingController(rootView: connectionView)
        self.connectionController = connectionController

        let connectionViewProxy = connectionController.view!
        connectionViewProxy.backgroundColor = .clear

        addChild(connectionController)
        connectionController.didMove(toParent: self)
        view.addConstrainedSubviews([activityIndicator, connectionViewProxy]) {
            connectionViewProxy.pinEdgesToSuperview(.all())
        }
    }

    private func hideSearchController() {
        searchController?.view.removeFromSuperview()
        searchController?.removeFromParent()

        searchController = nil
    }
}
