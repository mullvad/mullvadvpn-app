//
//  AppButton.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
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
    }

}
