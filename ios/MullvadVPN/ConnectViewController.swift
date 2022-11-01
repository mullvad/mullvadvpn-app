//
//  ConnectViewController.swift
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

protocol ConnectViewControllerDelegate: AnyObject {
    func connectViewControllerShouldShowSelectLocationPicker(_ controller: ConnectViewController)
}

class ConnectViewController: UIViewController, MKMapViewDelegate, RootContainment {
    private static let geoJSONSourceFileName = "countries.geo.json"
    private static let locationMarkerReuseIdentifier = "location"

    private let interactor: ConnectInteractor

    weak var delegate: ConnectViewControllerDelegate?

    let notificationController = NotificationController()

    private let contentView: ConnectContentView = {
        let view = ConnectContentView(frame: UIScreen.main.bounds)
        view.translatesAutoresizingMaskIntoConstraints = false
        return view
    }()

    private let logger = Logger(label: "ConnectViewController")

    private var targetRegion: MKCoordinateRegion?
    private let locationMarker = MKPointAnnotation()

    private var isAnimatingMap = false
    private var mapRegionAnimationDidEnd: (() -> Void)?

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

    private var tunnelState: TunnelState = .disconnected {
        didSet {
            setNeedsHeaderBarStyleAppearanceUpdate()
            updateTunnelRelay()
            updateUserInterfaceForTunnelStateChange()

            // Avoid unnecessary animations, particularly when this property is changed from inside
            // the `viewDidLoad`.
            let isViewVisible = viewIfLoaded?.window != nil

            updateLocation(animated: isViewVisible)
        }
    }

    init(interactor: ConnectInteractor) {
        self.interactor = interactor

        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        contentView.connectButton.addTarget(
            self,
            action: #selector(handleConnect(_:)),
            for: .touchUpInside
        )
        contentView.cancelButton.addTarget(
            self,
            action: #selector(handleDisconnect(_:)),
            for: .touchUpInside
        )
        contentView.splitDisconnectButton.primaryButton.addTarget(
            self,
            action: #selector(handleDisconnect(_:)),
            for: .touchUpInside
        )
        contentView.splitDisconnectButton.secondaryButton.addTarget(
            self,
            action: #selector(handleReconnect(_:)),
            for: .touchUpInside
        )

        contentView.selectLocationButton.addTarget(
            self,
            action: #selector(handleSelectLocation(_:)),
            for: .touchUpInside
        )

        interactor.didUpdateDeviceState = { [weak self] deviceState in
            self?.setNeedsHeaderBarStyleAppearanceUpdate()
        }

        interactor.didUpdateTunnelStatus = { [weak self] tunnelStatus in
            self?.tunnelState = tunnelStatus.state
        }

        tunnelState = interactor.tunnelStatus.state

        addSubviews()
        setupMapView()
        updateLocation(animated: false)
        addNotificationController()
    }

    override func traitCollectionDidChange(_ previousTraitCollection: UITraitCollection?) {
        super.traitCollectionDidChange(previousTraitCollection)

        if previousTraitCollection?.userInterfaceIdiom != traitCollection.userInterfaceIdiom ||
            previousTraitCollection?.horizontalSizeClass != traitCollection.horizontalSizeClass
        {
            updateTraitDependentViews()
        }
    }

    func setMainContentHidden(_ isHidden: Bool, animated: Bool) {
        let actions = {
            self.contentView.containerView.alpha = isHidden ? 0 : 1
        }

        if animated {
            UIView.animate(withDuration: 0.25, animations: actions)
        } else {
            actions()
        }
    }

    private func addSubviews() {
        view.addSubview(contentView)

        NSLayoutConstraint.activate([
            contentView.topAnchor.constraint(equalTo: view.topAnchor),
            contentView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            contentView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            contentView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])

        // Force layout since we rely on view frames when positioning map camera.
        view.layoutIfNeeded()
    }

    override func viewWillTransition(
        to size: CGSize,
        with coordinator: UIViewControllerTransitionCoordinator
    ) {
        super.viewWillTransition(to: size, with: coordinator)

        coordinator.animate(alongsideTransition: { _ in }, completion: { context in
            self.updateLocation(animated: context.isAnimated)
        })
    }

    // MARK: - Private

    private func updateUserInterfaceForTunnelStateChange() {
        contentView.secureLabel.text = tunnelState.localizedTitleForSecureLabel.uppercased()
        contentView.secureLabel.textColor = tunnelState.textColorForSecureLabel

        contentView.connectButton.setTitle(
            NSLocalizedString(
                "CONNECT_BUTTON_TITLE",
                tableName: "Main",
                value: "Secure connection",
                comment: ""
            ), for: .normal
        )
        contentView.selectLocationButton.setTitle(
            tunnelState.localizedTitleForSelectLocationButton,
            for: .normal
        )
        contentView.cancelButton.setTitle(
            NSLocalizedString(
                "CANCEL_BUTTON_TITLE",
                tableName: "Main",
                value: "Cancel",
                comment: ""
            ), for: .normal
        )
        contentView.splitDisconnectButton.primaryButton.setTitle(
            NSLocalizedString(
                "DISCONNECT_BUTTON_TITLE",
                tableName: "Main",
                value: "Disconnect",
                comment: ""
            ), for: .normal
        )
        contentView.splitDisconnectButton.secondaryButton.accessibilityLabel = NSLocalizedString(
            "RECONNECT_BUTTON_ACCESSIBILITY_LABEL",
            tableName: "Main",
            value: "Reconnect",
            comment: ""
        )

        updateTraitDependentViews()
    }

    private func updateTraitDependentViews() {
        contentView.setActionButtons(tunnelState.actionButtons(traitCollection: traitCollection))
    }

    private func attributedStringForLocation(string: String) -> NSAttributedString {
        let paragraphStyle = NSMutableParagraphStyle()
        paragraphStyle.lineSpacing = 0
        paragraphStyle.lineHeightMultiple = 0.80
        return NSAttributedString(string: string, attributes: [
            .paragraphStyle: paragraphStyle,
        ])
    }

    private func updateTunnelRelay() {
        switch tunnelState {
        case let .connecting(tunnelRelay):
            setTunnelRelay(tunnelRelay)

        case let .connected(tunnelRelay), let .reconnecting(tunnelRelay):
            setTunnelRelay(tunnelRelay)

        case .disconnected, .disconnecting, .pendingReconnect, .waitingForConnectivity:
            setTunnelRelay(nil)
        }

        contentView.locationContainerView.accessibilityLabel = tunnelState
            .localizedAccessibilityLabel
    }

    private func setTunnelRelay(_ tunnelRelay: PacketTunnelRelay?) {
        if let tunnelRelay = tunnelRelay {
            contentView.cityLabel
                .attributedText = attributedStringForLocation(string: tunnelRelay.location.city)
            contentView.countryLabel
                .attributedText = attributedStringForLocation(string: tunnelRelay.location.country)

            contentView.connectionPanel.dataSource = ConnectionPanelData(
                inAddress: "\(tunnelRelay.ipv4Relay) UDP",
                outAddress: nil
            )
            contentView.connectionPanel.isHidden = false
            contentView.connectionPanel.connectedRelayName = tunnelRelay.hostname
        } else {
            contentView.countryLabel.attributedText = attributedStringForLocation(string: " ")
            contentView.cityLabel.attributedText = attributedStringForLocation(string: " ")
            contentView.connectionPanel.dataSource = nil
            contentView.connectionPanel.isHidden = true
        }
    }

    private func locationMarkerOffset() -> CGPoint {
        // Compute the activity indicator frame within the view coordinate system.
        let activityIndicatorFrame = contentView.activityIndicator.convert(
            contentView.activityIndicator.bounds,
            to: view
        )

        // Compute the offset to align the marker on the map with activity indicator.
        let offsetY = activityIndicatorFrame.midY - contentView.mapView.frame.midY

        return CGPoint(x: 0, y: offsetY)
    }

    private func computeCoordinateRegion(
        center: CLLocationCoordinate2D,
        offset: CGPoint
    ) -> MKCoordinateRegion {
        let span = MKCoordinateSpan(latitudeDelta: 30, longitudeDelta: 30)
        var region = contentView.mapView
            .regionThatFits(MKCoordinateRegion(center: center, span: span))

        let latitudeDeltaPerPoint = region.span.latitudeDelta / contentView.mapView.frame.height
        region.center = center
        region.center.latitude += CLLocationDegrees(latitudeDeltaPerPoint * offset.y)

        return contentView.mapView.regionThatFits(region)
    }

    private func updateLocation(animated: Bool) {
        switch tunnelState {
        case let .connecting(tunnelRelay):
            removeLocationMarker()
            contentView.activityIndicator.startAnimating()

            if let tunnelRelay = tunnelRelay {
                setLocation(coordinate: tunnelRelay.location.geoCoordinate, animated: animated)
            } else {
                unsetLocation(animated: animated)
            }

        case let .reconnecting(tunnelRelay):
            removeLocationMarker()
            contentView.activityIndicator.startAnimating()

            setLocation(coordinate: tunnelRelay.location.geoCoordinate, animated: animated)

        case let .connected(tunnelRelay):
            // Show marker right away if activity indicator is not animating, i.e when the app
            // launches with connected tunnel.
            let showMarkerRightAway = !contentView.activityIndicator.isAnimating

            if showMarkerRightAway {
                addLocationMarker(coordinate: tunnelRelay.location.geoCoordinate)
            }

            setLocation(
                coordinate: tunnelRelay.location.geoCoordinate,
                animated: animated
            ) { [weak self] in
                if !showMarkerRightAway {
                    self?.contentView.activityIndicator.stopAnimating()
                    self?.addLocationMarker(coordinate: tunnelRelay.location.geoCoordinate)
                }
            }

        case .pendingReconnect:
            removeLocationMarker()
            contentView.activityIndicator.startAnimating()

        case .waitingForConnectivity:
            removeLocationMarker()

        case .disconnected, .disconnecting:
            removeLocationMarker()
            contentView.activityIndicator.stopAnimating()

            unsetLocation(animated: animated)
        }
    }

    private func addLocationMarker(coordinate: CLLocationCoordinate2D) {
        locationMarker.coordinate = coordinate
        contentView.mapView.addAnnotation(locationMarker)
    }

    private func removeLocationMarker() {
        contentView.mapView.removeAnnotation(locationMarker)
    }

    private func setLocation(
        coordinate: CLLocationCoordinate2D,
        animated: Bool,
        animationDidEnd: (() -> Void)? = nil
    ) {
        let markerOffset = locationMarkerOffset()
        let region = computeCoordinateRegion(center: coordinate, offset: markerOffset)

        if targetRegion?.isApproximatelyEqualTo(region) ?? false {
            if isAnimatingMap {
                mapRegionAnimationDidEnd = animationDidEnd
            } else {
                animationDidEnd?()
            }
            return
        }

        mapRegionAnimationDidEnd = animationDidEnd
        setMapRegion(region, animated: animated)
    }

    private func unsetLocation(animated: Bool) {
        let span = MKCoordinateSpan(latitudeDelta: 90, longitudeDelta: 90)
        let coordinate = CLLocationCoordinate2D(latitude: 0, longitude: 0)
        let region = contentView.mapView.regionThatFits(
            MKCoordinateRegion(center: coordinate, span: span)
        )

        mapRegionAnimationDidEnd = nil

        if targetRegion?.isApproximatelyEqualTo(region) ?? false {
            return
        }

        setMapRegion(region, animated: animated)
    }

    private func setMapRegion(_ region: MKCoordinateRegion, animated: Bool) {
        contentView.mapView.setRegion(region, animated: animated)
        isAnimatingMap = true
        targetRegion = region
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

    // MARK: - Actions

    @objc func handleConnect(_ sender: Any) {
        interactor.startTunnel()
    }

    @objc func handleDisconnect(_ sender: Any) {
        interactor.stopTunnel()
    }

    @objc func handleReconnect(_ sender: Any) {
        interactor.reconnectTunnel(selectNewRelay: true)
    }

    @objc func handleSelectLocation(_ sender: Any) {
        delegate?.connectViewControllerShouldShowSelectLocationPicker(self)
    }

    // MARK: - MKMapViewDelegate

    func mapView(_ mapView: MKMapView, rendererFor overlay: MKOverlay) -> MKOverlayRenderer {
        if let polygon = overlay as? MKPolygon {
            let renderer = MKPolygonRenderer(polygon: polygon)
            renderer.fillColor = .primaryColor
            renderer.strokeColor = .secondaryColor
            renderer.lineWidth = 1
            renderer.lineCap = .round
            renderer.lineJoin = .round
            return renderer
        }

        if let tileOverlay = overlay as? MKTileOverlay {
            return CustomOverlayRenderer(overlay: tileOverlay)
        }

        return MKOverlayRenderer()
    }

    func mapView(_ mapView: MKMapView, viewFor annotation: MKAnnotation) -> MKAnnotationView? {
        if annotation === locationMarker {
            let view = mapView.dequeueReusableAnnotationView(
                withIdentifier: Self.locationMarkerReuseIdentifier,
                for: annotation
            )
            view.isDraggable = false
            view.canShowCallout = false
            view.image = UIImage(named: "LocationMarkerSecure")
            return view
        }
        return nil
    }

    func mapView(_ mapView: MKMapView, regionDidChangeAnimated animated: Bool) {
        mapRegionAnimationDidEnd?()
        mapRegionAnimationDidEnd = nil
        isAnimatingMap = false
    }

    // MARK: - Private

    private func setupMapView() {
        contentView.mapView.insetsLayoutMarginsFromSafeArea = false
        contentView.mapView.delegate = self
        contentView.mapView.register(
            MKAnnotationView.self,
            forAnnotationViewWithReuseIdentifier: Self.locationMarkerReuseIdentifier
        )

        // Use dark style for the map to dim the map grid
        contentView.mapView.overrideUserInterfaceStyle = .dark

        addTileOverlay()
        loadGeoJSONData()
    }

    private func addTileOverlay() {
        // Use `nil` for template URL to make sure that Apple maps do not load
        // tiles from remote.
        let tileOverlay = MKTileOverlay(urlTemplate: nil)

        // Replace the default map tiles
        tileOverlay.canReplaceMapContent = true

        contentView.mapView.addOverlay(tileOverlay, level: .aboveLabels)
    }

    private func loadGeoJSONData() {
        guard let fileURL = Bundle.main.url(
            forResource: Self.geoJSONSourceFileName,
            withExtension: nil
        ) else {
            logger.debug("Failed to locate \(Self.geoJSONSourceFileName) in main bundle.")
            return
        }

        do {
            let data = try Data(contentsOf: fileURL)
            let overlays = try GeoJSON.decodeGeoJSON(data)

            contentView.mapView.addOverlays(overlays, level: .aboveLabels)
        } catch {
            logger.error(error: error, message: "Failed to load geojson.")
        }
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

    func actionButtons(traitCollection: UITraitCollection) -> [ConnectContentView.ActionButton] {
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

private extension MKCoordinateRegion {
    func isApproximatelyEqualTo(_ other: MKCoordinateRegion) -> Bool {
        return fabs(center.latitude - other.center.latitude) <= .ulpOfOne &&
            fabs(center.longitude - other.center.longitude) <= .ulpOfOne &&
            fabs(span.latitudeDelta - other.span.latitudeDelta) <= .ulpOfOne &&
            fabs(span.longitudeDelta - other.span.longitudeDelta) <= .ulpOfOne
    }
}
