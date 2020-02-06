//
//  EmbeddedViewContainerView.swift
//  MullvadVPN
//
//  Created by pronebird on 09/12/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

/// A `UIView` subclass that implements a host view for an embedded subview via an outlet.
@IBDesignable class EmbeddedViewContainerView: UIView {
    @IBOutlet var embeddedView: UIView!

    override func awakeFromNib() {
        super.awakeFromNib()

        backgroundColor = .clear

        embeddedView.translatesAutoresizingMaskIntoConstraints = false

        addSubview(embeddedView)

        NSLayoutConstraint.activate([
            embeddedView.topAnchor.constraint(equalTo: topAnchor),
            embeddedView.leadingAnchor.constraint(equalTo: leadingAnchor),
            embeddedView.trailingAnchor.constraint(equalTo: trailingAnchor),
            embeddedView.bottomAnchor.constraint(equalTo: bottomAnchor)
        ])
    }

    #if TARGET_INTERFACE_BUILDER
    override func draw(_ rect: CGRect) {
        UIColor.white.withAlphaComponent(0.3).setFill()
        UIColor.white.withAlphaComponent(0.6).setStroke()

        let bezierPath = UIBezierPath(rect: rect)
        bezierPath.lineWidth = 1
        bezierPath.fill()
        bezierPath.stroke()

        let attributedString = NSAttributedString(
            string: "UIView",
            attributes: [.foregroundColor: UIColor.white]
        )

        let textSize = attributedString.size()

        var textRect = rect
        textRect.origin.x = (rect.width - textSize.width) * 0.5
        textRect.origin.y = (rect.height - textSize.height) * 0.5
        textRect.size = textSize

        attributedString.draw(in: textRect)
    }
    #endif


}
