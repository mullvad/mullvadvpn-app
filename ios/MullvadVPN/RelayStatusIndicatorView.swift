//
//  RelayStatusIndicatorView.swift
//  MullvadVPN
//
//  Created by pronebird on 01/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import UIKit

@IBDesignable class RelayStatusIndicatorView: UIControl {

    private let circleLayer: CAShapeLayer = {
        let layer = CAShapeLayer()
        layer.needsDisplayOnBoundsChange = true
        return layer
    }()

    @IBInspectable var isActive: Bool = false {
        didSet {
            updateCircleLayerColor()
        }
    }

    override init(frame: CGRect) {
        super.init(frame: frame)

        setup()
    }

    required init?(coder aDecoder: NSCoder) {
        super.init(coder: aDecoder)

        setup()
    }

    private func setup() {
        backgroundColor = UIColor.clear

        layer.addSublayer(circleLayer)
        updateCircleLayerColor()
    }

    override var isHighlighted: Bool {
        didSet {
            updateCircleLayerColor()
        }
    }

    private func updateCircleLayerColor() {
        let baseColor = isActive
            ? UIColor.RelayStatusIndicator.activeColor
            : UIColor.RelayStatusIndicator.inactiveColor

        let circleColor = isHighlighted
            ? baseColor.darkened(by: 0.2) ?? baseColor
            : baseColor

        circleLayer.fillColor = circleColor.cgColor
    }

    override func layoutSublayers(of layer: CALayer) {
        super.layoutSublayers(of: layer)

        guard layer == self.layer else { return }

        // keep the circular layer square
        let shortSide = min(layer.bounds.width, layer.bounds.height)
        let circleOrigin = CGPoint(
            x: (layer.bounds.width - shortSide) * 0.5,
            y: (layer.bounds.height - shortSide) * 0.5
        )
        let circleSize = CGSize(width: shortSide, height: shortSide)
        let bezierPath = UIBezierPath(ovalIn: CGRect(origin: .zero, size: circleSize))

        circleLayer.frame = CGRect(origin: circleOrigin, size: circleSize)
        circleLayer.path = bezierPath.cgPath
    }
}
