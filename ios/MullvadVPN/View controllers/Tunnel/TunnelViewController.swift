//
//  TunnelViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import MapKit
import MullvadLogging
import MullvadTypes
import TunnelProviderMessaging
import UIKit

class TunnelViewController: UIViewController, RootContainment {
    private let logger = Logger(label: "TunnelViewController")
    private let interactor: TunnelViewControllerInteractor
    private let contentView = TunnelControlView(frame: CGRect(x: 0, y: 0, width: 320, height: 480))
    private var tunnelState: TunnelState = .disconnected

    var shouldShowSelectLocationPicker: (() -> Void)?

    let notificationController = NotificationController()
    private let mapViewController = MapViewController()

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
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
        return false
    }

    init(interactor: TunnelViewControllerInteractor) {
        self.interactor = interactor

        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        interactor.didUpdateDeviceState = { [weak self] deviceState in
            self?.setNeedsHeaderBarStyleAppearanceUpdate()
        }

        interactor.didUpdateTunnelStatus = { [weak self] tunnelStatus in
            self?.setTunnelState(tunnelStatus.state, animated: true)
        }

        contentView.actionHandler = { [weak self] action in
            switch action {
            case .connect:
                self?.interactor.startTunnel()

            case .disconnect, .cancel:
                self?.interactor.stopTunnel()

            case .reconnect:
                self?.interactor.reconnectTunnel(selectNewRelay: true)

            case .selectLocation:
                self?.shouldShowSelectLocationPicker?()
            }
        }

        addMapController()
        addContentView()
        addNotificationController()

        tunnelState = interactor.tunnelStatus.state
        updateContentView(animated: false)
        updateMap(animated: false)
    }

    override func viewWillTransition(
        to size: CGSize,
        with coordinator: UIViewControllerTransitionCoordinator
    ) {
        super.viewWillTransition(to: size, with: coordinator)

        coordinator.animate(alongsideTransition: nil, completion: { context in
            self.updateContentView(animated: context.isAnimated)
        })
    }

    func setMainContentHidden(_ isHidden: Bool, animated: Bool) {
        let actions = {
            self.contentView.alpha = isHidden ? 0 : 1
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

        updateContentView(animated: animated)
        updateMap(animated: animated)
    }

    private func updateMap(animated: Bool) {
        switch tunnelState {
        case let .connecting(tunnelRelay):
            mapViewController.removeLocationMarker()
            contentView.setAnimatingActivity(true)
            mapViewController.setCenter(tunnelRelay?.location.geoCoordinate, animated: animated)

        case let .reconnecting(tunnelRelay):
            mapViewController.removeLocationMarker()
            contentView.setAnimatingActivity(true)
            mapViewController.setCenter(tunnelRelay.location.geoCoordinate, animated: animated)

        case let .connected(tunnelRelay):
            let center = tunnelRelay.location.geoCoordinate

            mapViewController.setCenter(center, animated: animated) {
                self.contentView.setAnimatingActivity(false)
                self.mapViewController.addLocationMarker(coordinate: center)
            }

        case .pendingReconnect:
            mapViewController.removeLocationMarker()
            contentView.setAnimatingActivity(true)

        case .waitingForConnectivity:
            mapViewController.removeLocationMarker()
            contentView.setAnimatingActivity(false)

        case .disconnected, .disconnecting:
            mapViewController.removeLocationMarker()
            contentView.setAnimatingActivity(false)
            mapViewController.setCenter(nil, animated: animated)
        }
    }

    private func updateContentView(animated: Bool) {
        contentView.update(from: tunnelState, animated: animated)
    }

    private func addMapController() {
        let mapView = mapViewController.view!
        mapView.translatesAutoresizingMaskIntoConstraints = false
        mapViewController.alignmentView = contentView.mapCenterAlignmentView

        addChild(mapViewController)
        view.addSubview(mapView)
        mapViewController.didMove(toParent: self)

        NSLayoutConstraint.activate([
            mapView.topAnchor.constraint(equalTo: view.topAnchor),
            mapView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            mapView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            mapView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])
    }

    private func addNotificationController() {
        let notificationView = notificationController.view!
        notificationView.translatesAutoresizingMaskIntoConstraints = false

        addChild(notificationController)
        view.addSubview(notificationView)
        notificationController.didMove(toParent: self)

        NSLayoutConstraint.activate([
            notificationView.topAnchor.constraint(equalTo: view.topAnchor),
            notificationView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            notificationView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            notificationView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])
    }

    private func addContentView() {
        contentView.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(contentView)

        NSLayoutConstraint.activate([
            contentView.topAnchor.constraint(equalTo: view.topAnchor),
            contentView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            contentView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            contentView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])
    }
}
