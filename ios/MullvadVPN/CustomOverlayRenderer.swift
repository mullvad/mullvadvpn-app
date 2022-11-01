//
//  CustomOverlayRenderer.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import MapKit

class CustomOverlayRenderer: MKOverlayRenderer {
    override func draw(_ mapRect: MKMapRect, zoomScale: MKZoomScale, in context: CGContext) {
        let drawRect = rect(for: mapRect)
        context.setFillColor(UIColor.secondaryColor.cgColor)
        context.fill(drawRect)
    }
}
