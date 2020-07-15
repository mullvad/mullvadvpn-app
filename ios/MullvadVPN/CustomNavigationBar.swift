//
//  CustomNavigationBar.swift
//  MullvadVPN
//
//  Created by pronebird on 22/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

class CustomNavigationBar: UINavigationBar {

    override init(frame: CGRect) {
        super.init(frame: frame)

        commonInit()
    }

    required init?(coder aDecoder: NSCoder) {
        super.init(coder: aDecoder)

        commonInit()
    }

    private func commonInit() {
        var margins = layoutMargins
        margins.left = 24
        margins.right = 24
        layoutMargins = margins

        if #available(iOS 13, *) {
            // no-op
        } else {
            barTintColor = .secondaryColor
            shadowImage = UIImage()
            isTranslucent = false
        }
    }

}
