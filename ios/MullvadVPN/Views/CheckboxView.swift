//
//  CheckboxView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

class CheckboxView: UIView {
    private let checkboxSelectedView: UIImageView = {
        UIImageView(image: UIImage.checkboxSelected)
    }()

    private let checkboxUnselectedView: UIImageView = {
        UIImageView(image: UIImage.checkboxUnselected)
    }()

    var isChecked = false {
        didSet {
            checkboxSelectedView.alpha = isChecked ? 1 : 0
        }
    }

    init() {
        super.init(frame: .zero)

        addConstrainedSubviews([checkboxSelectedView, checkboxUnselectedView]) {
            checkboxSelectedView.pinEdgesToSuperview()
            checkboxUnselectedView.pinEdgesToSuperview()
        }
    }

    required init?(coder aDecoder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
