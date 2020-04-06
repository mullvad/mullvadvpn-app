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

            setNeedsLayout()
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

        // Make sure the buy button scales down the font size to fit the long labels.
        // Changing baseline adjustment helps to prevent the text from being misaligned after
        // being scaled down.
        titleLabel?.adjustsFontSizeToFitWidth = true
        titleLabel?.baselineAdjustment = .alignCenters
    }

    override func layoutSubviews() {
        super.layoutSubviews()

        activityIndicator.frame = activityIndicatorRect(
            forContentRect: contentRect(forBounds: bounds)
        )
    }

    override func titleRect(forContentRect contentRect: CGRect) -> CGRect {
        var titleRect = super.titleRect(forContentRect: contentRect)
        let activityIndicatorRect = self.activityIndicatorRect(forContentRect: contentRect)

        // Adjust the title frame in case if it overlaps the activity indicator
        let intersection = titleRect.intersection(activityIndicatorRect)
        if !intersection.isNull {
            if case .leftToRight = effectiveUserInterfaceLayoutDirection {
                titleRect.origin.x = max(contentRect.minX, titleRect.minX - intersection.width)
                titleRect.size.width = intersection.minX - titleRect.minX
            } else {
                titleRect.origin.x = titleRect.minX + intersection.width
                titleRect.size.width = min(contentRect.maxX, titleRect.maxX) - intersection.maxX
            }
        }

        return titleRect
    }

    private func activityIndicatorRect(forContentRect contentRect: CGRect) -> CGRect {
        var frame = activityIndicator.frame

        if case .leftToRight = effectiveUserInterfaceLayoutDirection {
            frame.origin.x = contentRect.maxX - frame.width
        } else {
            frame.origin.x = contentRect.minX
        }

        frame.origin.y = contentRect.midY - frame.height * 0.5

        return frame
    }
}
