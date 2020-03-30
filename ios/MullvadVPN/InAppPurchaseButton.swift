//
//  InAppPurchaseButton.swift
//  MullvadVPN
//
//  Created by pronebird on 23/03/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class InAppPurchaseButton: AppButton {

    let activityIndicator = SpinnerActivityIndicatorView(style: .medium)

    var isLoading: Bool = false {
        didSet {
            if isLoading {
                activityIndicator.startAnimating()
            } else {
                activityIndicator.stopAnimating()
            }

            titleLabel?.alpha = isLoading ? 0 : 1
        }
    }

    override init(frame: CGRect) {
        super.init(frame: frame)
        commonInit()
    }

    required init?(coder aDecoder: NSCoder) {
        super.init(coder: aDecoder)
        commonInit()
    }

    private func commonInit() {
        addSubview(activityIndicator)
    }

    override func layoutSubviews() {
        super.layoutSubviews()

        activityIndicator.center = self.center
    }
}
