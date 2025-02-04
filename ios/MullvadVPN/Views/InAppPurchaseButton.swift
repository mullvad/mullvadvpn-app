//
//  InAppPurchaseButton.swift
//  MullvadVPN
//
//  Created by pronebird on 23/03/2020.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class InAppPurchaseButton: AppButton {
    let activityIndicator = SpinnerActivityIndicatorView(style: .medium)

    var isLoading = false {
        didSet {
            if isLoading {
                activityIndicator.startAnimating()
            } else {
                activityIndicator.stopAnimating()
            }

            setNeedsLayout()
        }
    }

    init() {
        super.init(style: .success)

        addSubview(activityIndicator)

        // Make sure the buy button scales down the font size to fit the long labels.
        // Changing baseline adjustment helps to prevent the text from being misaligned after
        // being scaled down.
        titleLabel?.adjustsFontSizeToFitWidth = true
        titleLabel?.baselineAdjustment = .alignCenters
    }

    required init?(coder aDecoder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func layoutSubviews() {
        super.layoutSubviews()

        // Calculate the content size after insets
        let contentSize = frame
        let contentEdgeInsets = configuration?.contentInsets ?? .zero
        let finalWidth = contentSize.width - (contentEdgeInsets.leading + contentEdgeInsets.trailing)
        let finalHeight = contentSize.height - (contentEdgeInsets.top + contentEdgeInsets.bottom)
        let contentRect = CGRect(
            origin: frame.origin,
            size: CGSize(width: finalWidth, height: finalHeight)
        )
        self.titleLabel?.frame = getTitleRect(forContentRect: contentRect)
        self.activityIndicator.frame = activityIndicatorRect(forContentRect: contentRect)
    }

    private func getTitleRect(forContentRect contentRect: CGRect) -> CGRect {
        var titleRect = titleLabel?.frame ?? .zero
        let activityIndicatorRect = activityIndicatorRect(forContentRect: contentRect)

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

        frame.origin.y = contentRect.midY
        return frame
    }
}
