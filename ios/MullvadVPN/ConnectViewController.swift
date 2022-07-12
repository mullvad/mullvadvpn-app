//
//  ConnectViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit
import MapKit
import Logging

class CustomOverlayRenderer: MKOverlayRenderer {
    override func draw(_ mapRect: MKMapRect, zoomScale: MKZoomScale, in context: CGContext) {
        let drawRect = self.rect(for: mapRect)
        context.setFillColor(UIColor.secondaryColor.cgColor)
        context.fill(drawRect)
    }
}

protocol ConnectViewControllerDelegate: AnyObject {
    func connectViewControllerShouldShowSelectLocationPicker(_ controller: ConnectViewController)
}

class ConnectViewController: UIViewController, MKMapViewDelegate, RootContainment, TunnelObserver {

    private static let geoJSONSourceFileName = "countries.geo.json"

    weak var delegate: ConnectViewControllerDelegate?

    let notificationController = NotificationController()

    private let mainContentView: ConnectMainContentView = {
        let view = ConnectMainContentView(frame: UIScreen.main.bounds)
        view.translatesAutoresizingMaskIntoConstraints = false
        return view
    }()

    private let logger = Logger(label: "ConnectViewController")

    private var lastLocation: CLLocationCoordinate2D?
    private let locationMarker = MKPointAnnotation()

    private var mapRegionAnimationDidEnd: (() -> Void)?

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    var preferredHeaderBarPresentation: HeaderBarPresentation {
        guard TunnelManager.shared.isAccountSet else {
            return HeaderBarPresentation(style: .default, showsDivider: true)
        }

        switch tunnelState {
        case .connecting, .reconnecting, .connected:
            return HeaderBarPresentation(style: .secured, showsDivider: false)

        case .disconnecting, .disconnected, .pendingReconnect:
            return HeaderBarPresentation(style: .unsecured, showsDivider: false)
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
            let isViewVisible = self.viewIfLoaded?.window != nil

            updateLocation(animated: isViewVisible)
        }
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        mainContentView.connectButton.addTarget(self, action: #selector(handleConnect(_:)), for: .touchUpInside)
        mainContentView.cancelButton.addTarget(self, action: #selector(handleDisconnect(_:)), for: .touchUpInside)
        mainContentView.splitDisconnectButton.primaryButton.addTarget(self, action: #selector(handleDisconnect(_:)), for: .touchUpInside)
        mainContentView.splitDisconnectButton.secondaryButton.addTarget(self, action: #selector(handleReconnect(_:)), for: .touchUpInside)

        mainContentView.selectLocationButton.addTarget(self, action: #selector(handleSelectLocation(_:)), for: .touchUpInside)

        TunnelManager.shared.addObserver(self)
        self.tunnelState = TunnelManager.shared.tunnelState

        addSubviews()
        setupMapView()
        updateLocation(animated: false)
        addNotificationController()

        TunnelManager.shared.addObserver(self)
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

    func tunnelManagerDidLoadConfiguration(_ manager: TunnelManager) {
        // no-op
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelSettings tunnelSettings: TunnelSettingsV2?) {
        setNeedsHeaderBarStyleAppearanceUpdate()
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelState tunnelState: TunnelState) {
        self.tunnelState = tunnelState
    }

    func tunnelManager(_ manager: TunnelManager, didFailWithError error: TunnelManager.Error) {
        // no-op
    }

    // MARK: - Private

    private func updateUserInterfaceForTunnelStateChange() {
        mainContentView.secureLabel.text = tunnelState.localizedTitleForSecureLabel.uppercased()
        mainContentView.secureLabel.textColor = tunnelState.textColorForSecureLabel

        mainContentView.connectButton.setTitle(
            NSLocalizedString(
                "CONNECT_BUTTON_TITLE",
                tableName: "Main",
                value: "Secure connection",
                comment: ""
            ), for: .normal
        )
        mainContentView.selectLocationButton.setTitle(tunnelState.localizedTitleForSelectLocationButton, for: .normal)
        mainContentView.cancelButton.setTitle(
            NSLocalizedString(
                "CANCEL_BUTTON_TITLE",
                tableName: "Main",
                value: "Cancel",
                comment: ""
            ), for: .normal)
        mainContentView.splitDisconnectButton.primaryButton.setTitle(
            NSLocalizedString(
                "DISCONNECT_BUTTON_TITLE",
                tableName: "Main",
                value: "Disconnect",
                comment: ""
            ), for: .normal
        )
        mainContentView.splitDisconnectButton.secondaryButton.accessibilityLabel = NSLocalizedString(
            "RECONNECT_BUTTON_ACCESSIBILITY_LABEL",
            tableName: "Main",
            value: "Reconnect",
            comment: ""
        )

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

    private func updateTunnelRelay() {
        switch tunnelState {
        case .connecting(let tunnelRelay):
            setTunnelRelay(tunnelRelay)

        case .connected(let tunnelRelay), .reconnecting(let tunnelRelay):
            setTunnelRelay(tunnelRelay)

        case .disconnected, .disconnecting, .pendingReconnect:
            setTunnelRelay(nil)
        }

        mainContentView.locationContainerView.accessibilityLabel = tunnelState.localizedAccessibilityLabel
    }

    private func setTunnelRelay(_ tunnelRelay: PacketTunnelRelay?) {
        if let tunnelRelay = tunnelRelay {
            mainContentView.cityLabel.attributedText = attributedStringForLocation(string: tunnelRelay.location.city)
            mainContentView.countryLabel.attributedText = attributedStringForLocation(string: tunnelRelay.location.country)

            mainContentView.connectionPanel.dataSource = ConnectionPanelData(
                inAddress: "\(tunnelRelay.ipv4Relay) UDP",
                outAddress: nil
            )
            mainContentView.connectionPanel.isHidden = false
            mainContentView.connectionPanel.connectedRelayName = tunnelRelay.hostname
        } else {
            mainContentView.countryLabel.attributedText = attributedStringForLocation(string: " ")
            mainContentView.cityLabel.attributedText = attributedStringForLocation(string: " ")
            mainContentView.connectionPanel.dataSource = nil
            mainContentView.connectionPanel.isHidden = true
        }
    }

    private func locationMarkerOffset() -> CGPoint {
        // Compute the activity indicator frame within the view coordinate system.
        let activityIndicatorFrame = mainContentView.activityIndicator.convert(mainContentView.activityIndicator.bounds, to: view)

        // Compute the offset to align the marker on the map with activity indicator.
        let offsetY = activityIndicatorFrame.midY - mainContentView.mapView.frame.midY

        return CGPoint(x: 0, y: offsetY)
    }

    private func computeCoordinateRegion(centerCoordinate: CLLocationCoordinate2D, centerOffsetInPoints: CGPoint) -> MKCoordinateRegion  {
        let span = MKCoordinateSpan(latitudeDelta: 30, longitudeDelta: 30)
        var region = MKCoordinateRegion(center: centerCoordinate, span: span)
        region = mainContentView.mapView.regionThatFits(region)

        let latitudeDeltaPerPoint = region.span.latitudeDelta / Double(mainContentView.mapView.frame.height)
        var offsetCenter = centerCoordinate
        offsetCenter.latitude += CLLocationDegrees(latitudeDeltaPerPoint * Double(centerOffsetInPoints.y))
        region.center = offsetCenter

        return region
    }

    private func updateLocation(animated: Bool) {
        switch tunnelState {
        case .connecting(let tunnelRelay):
            removeLocationMarker()
            mainContentView.activityIndicator.startAnimating()

            if let tunnelRelay = tunnelRelay {
                setLocation(coordinate: tunnelRelay.location.geoCoordinate, animated: animated)
            } else {
                unsetLocation(animated: animated)
            }

        case .reconnecting(let tunnelRelay):
            removeLocationMarker()
            mainContentView.activityIndicator.startAnimating()

            setLocation(coordinate: tunnelRelay.location.geoCoordinate, animated: animated)

        case .connected(let tunnelRelay):
            setLocation(coordinate: tunnelRelay.location.geoCoordinate, animated: animated) { [weak self] in
                self?.mainContentView.activityIndicator.stopAnimating()
                self?.addLocationMarker(coordinate: tunnelRelay.location.geoCoordinate)
            }

        case .pendingReconnect:
            removeLocationMarker()
            mainContentView.activityIndicator.startAnimating()

        case .disconnected, .disconnecting:
            removeLocationMarker()
            mainContentView.activityIndicator.stopAnimating()

            unsetLocation(animated: animated)
        }
    }

    private func addLocationMarker(coordinate: CLLocationCoordinate2D) {
        locationMarker.coordinate = coordinate
        mainContentView.mapView.addAnnotation(locationMarker)
    }

    private func removeLocationMarker() {
        mainContentView.mapView.removeAnnotation(locationMarker)
    }

    private func setLocation(coordinate: CLLocationCoordinate2D, animated: Bool, animationDidEnd: (() -> Void)? = nil) {
        if let lastLocation = lastLocation, coordinate.approximatelyEqualTo(lastLocation) {
            mapRegionAnimationDidEnd = nil
            animationDidEnd?()
            return
        }

        mapRegionAnimationDidEnd = animationDidEnd

        let markerOffset = locationMarkerOffset()
        let region = computeCoordinateRegion(centerCoordinate: coordinate, centerOffsetInPoints: markerOffset)

        mainContentView.mapView.setRegion(region, animated: animated)

        self.lastLocation = coordinate
    }

    private func unsetLocation(animated: Bool, animationDidEnd: (() -> Void)? = nil) {
        let coordinate = CLLocationCoordinate2D(latitude: 0, longitude: 0)
        if let lastLocation = lastLocation, coordinate.approximatelyEqualTo(lastLocation) {
            mapRegionAnimationDidEnd = nil
            animationDidEnd?()
            return
        }

        mapRegionAnimationDidEnd = animationDidEnd

        let span = MKCoordinateSpan(latitudeDelta: 90, longitudeDelta: 90)
        let region = MKCoordinateRegion(center: coordinate, span: span)
        mainContentView.mapView.setRegion(region, animated: animated)

        self.lastLocation = coordinate
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
            notificationView.bottomAnchor.constraint(equalTo: view.bottomAnchor)
        ])
    }

    // MARK: - Actions

    @objc func handleConnect(_ sender: Any) {
        TunnelManager.shared.startTunnel()
    }

    @objc func handleDisconnect(_ sender: Any) {
        TunnelManager.shared.stopTunnel()
    }

    @objc func handleReconnect(_ sender: Any) {
        TunnelManager.shared.reconnectTunnel()
    }

    @objc func handleSelectLocation(_ sender: Any) {
        delegate?.connectViewControllerShouldShowSelectLocationPicker(self)
    }

    // MARK: - MKMapViewDelegate

    func mapView(_ mapView: MKMapView, rendererFor overlay: MKOverlay) -> MKOverlayRenderer {
        if let polygon = overlay as? MKPolygon {
            let renderer = MKPolygonRenderer(polygon: polygon)
            renderer.fillColor = UIColor.primaryColor
            renderer.strokeColor = UIColor.secondaryColor
            renderer.lineWidth = 1.0
            renderer.lineCap = .round
            renderer.lineJoin = .round

            return renderer
        }

        if let tileOverlay = overlay as? MKTileOverlay {
            return CustomOverlayRenderer(overlay: tileOverlay)
        }

        fatalError()
    }

    func mapView(_ mapView: MKMapView, viewFor annotation: MKAnnotation) -> MKAnnotationView? {
        if annotation === locationMarker {
            let view = mapView.dequeueReusableAnnotationView(withIdentifier: "location", for: annotation)
            view.isDraggable = false
            view.canShowCallout = false
            view.image = self.locationMarkerSecureImage
            return view
        }
        return nil
    }

    func mapView(_ mapView: MKMapView, regionDidChangeAnimated animated: Bool) {
        mapRegionAnimationDidEnd?()
        mapRegionAnimationDidEnd = nil
    }

    // MARK: - Private

    private var locationMarkerSecureImage: UIImage {
        return UIImage(named: "LocationMarkerSecure")!
    }

    private func setupMapView() {
        mainContentView.mapView.insetsLayoutMarginsFromSafeArea = false
        mainContentView.mapView.delegate = self
        mainContentView.mapView.register(MKAnnotationView.self, forAnnotationViewWithReuseIdentifier: "location")

        if #available(iOS 13.0, *) {
            // Use dark style for the map to dim the map grid
            mainContentView.mapView.overrideUserInterfaceStyle = .dark
        }

        addTileOverlay()
        loadGeoJSONData()
    }

    private func addTileOverlay() {
        // Use `nil` for template URL to make sure that Apple maps do not load
        // tiles from remote.
        let tileOverlay = MKTileOverlay(urlTemplate: nil)

        // Replace the default map tiles
        tileOverlay.canReplaceMapContent = true

        mainContentView.mapView.addOverlay(tileOverlay)
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

            mainContentView.mapView.addOverlays(overlays)
        } catch {
            logger.error(chainedError: AnyChainedError(error), message: "Failed to load geojson.")
        }
    }
}

private extension TunnelState {

    var textColorForSecureLabel: UIColor {
        switch self {
        case .connecting, .reconnecting:
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
        case .connecting, .connected, .reconnecting:
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

        case .connected(let tunnelInfo):
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

        case .reconnecting(let tunnelInfo):
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

    func actionButtons(traitCollection: UITraitCollection) -> [ConnectMainContentView.ActionButton] {
        switch (traitCollection.userInterfaceIdiom, traitCollection.horizontalSizeClass) {
        case (.phone, _), (.pad, .compact):
            switch self {
            case .disconnected, .disconnecting(.nothing):
                return [.selectLocation, .connect]

            case .connecting, .pendingReconnect, .disconnecting(.reconnect):
                return [.selectLocation, .cancel]

            case .connected, .reconnecting:
                return [.selectLocation, .disconnect]
            }

        case (.pad, .regular):
            switch self {
            case .disconnected, .disconnecting(.nothing):
                return [.connect]

            case .connecting, .pendingReconnect, .disconnecting(.reconnect):
                return [.cancel]

            case .connected, .reconnecting:
                return [.disconnect]
            }

        default:
            return []
        }
    }

}

private extension CLLocationCoordinate2D {
    func approximatelyEqualTo(_ other: CLLocationCoordinate2D) -> Bool {
        return fabs(self.latitude - other.latitude) <= .ulpOfOne &&
            fabs(self.longitude - other.longitude) <= .ulpOfOne
    }
}
