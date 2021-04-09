//
//  CustomNavigationBar.swift
//  MullvadVPN
//
//  Created by pronebird on 22/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

class CustomNavigationBar: UINavigationBar {

    private static let setupAppearanceOnce: Void = {
        let buttonAppearance = UIBarButtonItem.appearance(whenContainedInInstancesOf: [CustomNavigationBar.self])
        buttonAppearance.setBackButtonTitlePositionAdjustment(UIOffset(horizontal: 4, vertical: 0), for: .default)
    }()

    var prefersOpaqueBackground: Bool {
        didSet {
            setOpaqueBackgroundAppearance(prefersOpaqueBackground)
        }
    }

    // Returns the distance from the title label to the bottom of navigation bar
    var titleLabelBottomInset: CGFloat {
        // Go two levels deep only
        let subviewsToExamine = subviews.flatMap { (view) -> [UIView] in
            return [view] + view.subviews
        }

        let titleLabel = subviewsToExamine.first { (view) -> Bool in
            return view is UILabel
        }

        if let titleLabel = titleLabel {
            let titleFrame = titleLabel.convert(titleLabel.bounds, to: self)
            return max(bounds.maxY - titleFrame.maxY, 0)
        } else {
            return 0
        }
    }

    override init(frame: CGRect) {
        Self.setupAppearanceOnce

        if #available(iOS 13, *) {
            prefersOpaqueBackground = false
        } else {
            prefersOpaqueBackground = true
        }

        super.init(frame: frame)

        var margins = layoutMargins
        margins.left = UIMetrics.contentLayoutMargins.left
        margins.right = UIMetrics.contentLayoutMargins.right
        layoutMargins = margins

        backIndicatorImage = UIImage(named: "IconBack")
        backIndicatorTransitionMaskImage = UIImage(named: "IconBackTransitionMask")

        setOpaqueBackgroundAppearance(prefersOpaqueBackground)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func setOpaqueBackgroundAppearance(_ flag: Bool) {
        if flag {
            barTintColor = .secondaryColor
            backgroundColor = .secondaryColor
            shadowImage = UIImage()
            isTranslucent = false
        } else {
            barTintColor = nil
            backgroundColor = nil
            shadowImage = nil
            isTranslucent = true
        }
    }

}
