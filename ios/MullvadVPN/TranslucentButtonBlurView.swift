//
//  TranslucentButtonBlurView.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

private let kButtonCornerRadius = CGFloat(4)

@IBDesignable class TranslucentButtonBlurView: UIVisualEffectView {

    override init(effect: UIVisualEffect?) {
        super.init(effect: effect)

        setup()
    }

    required init?(coder aDecoder: NSCoder) {
        super.init(coder: aDecoder)

        setup()
    }

    private func setup() {
        layer.cornerRadius = kButtonCornerRadius
        layer.masksToBounds = true

        updateCornerMask()
    }

    override func awakeFromNib() {
        super.awakeFromNib()

        updateCornerMask()
    }

    private func updateCornerMask() {
        for case let button as AppButton in contentView.subviews {
            layer.maskedCorners = Self.cornerMask(buttonStyle: button.style)
        }
    }

    private class func cornerMask(buttonStyle: AppButton.Style) -> CACornerMask {
        switch buttonStyle {
        case .translucentDangerSplitLeft:
            return [.layerMinXMinYCorner, .layerMinXMaxYCorner]
        case .translucentDangerSplitRight:
            return [.layerMaxXMinYCorner, .layerMaxXMaxYCorner]
        default:
            return [
                .layerMinXMinYCorner, .layerMinXMaxYCorner,
                .layerMaxXMinYCorner, .layerMaxXMaxYCorner
            ]
        }
    }

}
