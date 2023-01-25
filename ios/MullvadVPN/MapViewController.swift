//
//  MapViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 03/01/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MapKit
import MullvadLogging
import Operations

private let locationMarkerReuseIdentifier = "location"
private let geoJSONSourceFileName = "countries.geo.json"

final class MapViewController: UIViewController, MKMapViewDelegate {
    private let logger = Logger(label: "MapViewController")
    private let animationQueue = AsyncOperationQueue.makeSerial()

    private let locationMarker = MKPointAnnotation()
    private var willChangeRegion = false
    private var regionDidChangeCompletion: (() -> Void)?
    private let mapView = MKMapView()
    private var isFirstLayoutPass = true
    private var center: CLLocationCoordinate2D?
    var alignmentView: UIView?

    // MARK: - View lifecycle

    override func viewDidLoad() {
        super.viewDidLoad()

        mapView.delegate = self
        mapView.register(
            MKAnnotationView.self,
            forAnnotationViewWithReuseIdentifier: locationMarkerReuseIdentifier
        )

        mapView.showsUserLocation = false
        mapView.isZoomEnabled = false
        mapView.isScrollEnabled = false
        mapView.isUserInteractionEnabled = false
        mapView.accessibilityElementsHidden = true

        // Use dark style for the map to dim the map grid
        mapView.overrideUserInterfaceStyle = .dark

        addTileOverlay()
        loadGeoJSONData()
        addMapView()
    }

    override func viewWillTransition(
        to size: CGSize,
        with coordinator: UIViewControllerTransitionCoordinator
    ) {
        super.viewWillTransition(to: size, with: coordinator)

        coordinator.animate(alongsideTransition: nil, completion: { context in
            self.recomputeVisibleRegion(animated: context.isAnimated)
        })
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()

        if isFirstLayoutPass {
            isFirstLayoutPass = false
            recomputeVisibleRegion(animated: false)
        }
    }

    // MARK: - Public

    func addLocationMarker(coordinate: CLLocationCoordinate2D) {
        locationMarker.coordinate = coordinate
        mapView.addAnnotation(locationMarker)
    }

    func removeLocationMarker() {
        mapView.removeAnnotation(locationMarker)
    }

    func setCenter(
        _ center: CLLocationCoordinate2D?,
        animated: Bool,
        completion: (() -> Void)? = nil
    ) {
        enqueueAnimation(cancelOtherAnimations: true) { finish in
            self.setCenterInternal(center, animated: animated) {
                finish()
                completion?()
            }
        }
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
        guard annotation === locationMarker else { return nil }

        let view = mapView.dequeueReusableAnnotationView(
            withIdentifier: locationMarkerReuseIdentifier,
            for: annotation
        )
        view.isDraggable = false
        view.canShowCallout = false
        view.image = UIImage(named: "LocationMarkerSecure")

        return view
    }

    func mapView(_ mapView: MKMapView, regionWillChangeAnimated animated: Bool) {
        willChangeRegion = true
    }

    func mapView(_ mapView: MKMapView, regionDidChangeAnimated animated: Bool) {
        willChangeRegion = false

        let handler = regionDidChangeCompletion
        regionDidChangeCompletion = nil
        handler?()
    }

    // MARK: - Private

    private func addMapView() {
        mapView.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(mapView)

        NSLayoutConstraint.activate([
            mapView.topAnchor.constraint(equalTo: view.topAnchor),
            mapView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            mapView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            mapView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])
    }

    private func addTileOverlay() {
        let tileOverlay = MKTileOverlay(urlTemplate: nil)
        tileOverlay.canReplaceMapContent = true

        mapView.addOverlay(tileOverlay, level: .aboveLabels)
    }

    private func loadGeoJSONData() {
        guard let fileURL = Bundle.main.url(forResource: geoJSONSourceFileName, withExtension: nil)
        else {
            logger.debug("Failed to locate \(geoJSONSourceFileName) in main bundle.")
            return
        }

        do {
            let data = try Data(contentsOf: fileURL)
            let overlays = try GeoJSON.decodeGeoJSON(data)

            mapView.addOverlays(overlays, level: .aboveLabels)
        } catch {
            logger.error(error: error, message: "Failed to load geojson.")
        }
    }

    private func setCenterInternal(
        _ center: CLLocationCoordinate2D?,
        animated: Bool,
        completion: (() -> Void)?
    ) {
        let region = makeRegion(center: center)

        self.center = center

        // Map view does not call delegate methods when attempting to set the same region.
        mapView.setRegion(region, animated: animated)

        if willChangeRegion {
            regionDidChangeCompletion = completion
        } else {
            completion?()
        }
    }

    private func recomputeVisibleRegion(animated: Bool) {
        enqueueAnimation(cancelOtherAnimations: false) { finish in
            self.setCenterInternal(self.center, animated: animated, completion: finish)
        }
    }

    private func enqueueAnimation(
        cancelOtherAnimations: Bool,
        block: @escaping (_ finish: @escaping () -> Void) -> Void
    ) {
        let operation = AsyncBlockOperation(dispatchQueue: .main) { operation in
            block {
                operation.finish()
            }
        }

        if cancelOtherAnimations {
            animationQueue.cancelAllOperations()
        }

        animationQueue.addOperation(operation)
    }

    private func makeRegion(center: CLLocationCoordinate2D?) -> MKCoordinateRegion {
        guard let center = center else {
            return makeZoomedOutRegion()
        }

        let sourceRegion = makeZoomedInRegion(center: center)

        guard let alignmentView = alignmentView else {
            return sourceRegion
        }

        return makeRegion(from: sourceRegion, withCenterMatching: alignmentView)
    }

    private func makeZoomedInRegion(center: CLLocationCoordinate2D) -> MKCoordinateRegion {
        let span = MKCoordinateSpan(latitudeDelta: 30, longitudeDelta: 30)
        let region = MKCoordinateRegion(center: center, span: span)

        return mapView.regionThatFits(region)
    }

    private func makeZoomedOutRegion() -> MKCoordinateRegion {
        let coordinate = CLLocationCoordinate2D(latitude: 0, longitude: 0)
        let span = MKCoordinateSpan(latitudeDelta: 90, longitudeDelta: 90)
        let region = MKCoordinateRegion(center: coordinate, span: span)

        return mapView.regionThatFits(region)
    }

    private func makeRegion(
        from region: MKCoordinateRegion,
        withCenterMatching alignmentView: UIView
    ) -> MKCoordinateRegion {
        // Map view center lies within layout margins frame.
        let mapViewLayoutFrame = mapView.layoutMarginsGuide.layoutFrame

        guard mapViewLayoutFrame.width > 0, mapView.frame.width > 0,
              region.span.longitudeDelta > 0,
              mapView.region.span.longitudeDelta > 0 else { return region }

        // MKMapView.convert(_:toRectTo:) returns CGRect scaled to the zoom level derived from
        // currently set region.
        // Calculate the ratio that we can use to translate the rect within its own coordinate
        // system before converting it into MKCoordinateRegion.
        let newZoomLevel = mapViewLayoutFrame.width / region.span.longitudeDelta
        let currentZoomLevel = mapViewLayoutFrame.width / mapView.region.span.longitudeDelta
        let zoomDelta = currentZoomLevel / newZoomLevel

        let alignmentViewRect = alignmentView.convert(alignmentView.bounds, to: mapView)
        let horizontalOffset = (mapViewLayoutFrame.midX - alignmentViewRect.midX) * zoomDelta
        let verticalOffset = (mapViewLayoutFrame.midY - alignmentViewRect.midY) * zoomDelta

        let regionRect = mapView.convert(region, toRectTo: mapView)
        let offsetRegionRect = regionRect.offsetBy(dx: horizontalOffset, dy: verticalOffset)
        let offsetRegion = mapView.convert(offsetRegionRect, toRegionFrom: mapView)

        if CLLocationCoordinate2DIsValid(offsetRegion.center) {
            return offsetRegion
        } else {
            return region
        }
    }
}
