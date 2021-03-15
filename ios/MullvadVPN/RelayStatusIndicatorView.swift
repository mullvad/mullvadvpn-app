//
//  RelayStatusIndicatorView.swift
//  MullvadVPN
//
//  Created by pronebird on 01/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

@IBDesignable class RelayStatusIndicatorView: UIControl {

    private let circleLayer: CALayer = {
        let layer = CALayer()
        layer.needsDisplayOnBoundsChange = true
        return layer
    }()

    @IBInspectable var isActive: Bool = false {
        didSet {
            updateCircleLayerColor()
        }
    }

    override var isHighlighted: Bool {
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

    override func tintColorDidChange() {
        super.tintColorDidChange()

        updateCircleLayerColor()
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

        circleLayer.frame = CGRect(origin: circleOrigin, size: circleSize)
        circleLayer.cornerRadius = shortSide * 0.5
    }

    private func setup() {
        isUserInteractionEnabled = false
        backgroundColor = UIColor.clear

        layer.addSublayer(circleLayer)
        updateCircleLayerColor()
    }

    private func updateCircleLayerColor() {
        let baseColor = isActive
            ? UIColor.RelayStatusIndicator.activeColor
            : UIColor.RelayStatusIndicator.inactiveColor

        let circleColor: UIColor = isHighlighted ? tintColor : baseColor

        CATransaction.begin()
        CATransaction.setDisableActions(true)
        circleLayer.backgroundColor = circleColor.cgColor
        CATransaction.commit()
    }
}
