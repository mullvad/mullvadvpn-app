//
//  CustomNavigationBar.swift
//  MullvadVPN
//
//  Created by pronebird on 22/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UINavigationBar {
    var titleLabelBottomInset: CGFloat {
        // Go two levels deep only
        let subviewsToExamine = subviews.flatMap { view -> [UIView] in
            return [view] + view.subviews
        }

        let titleLabel = subviewsToExamine.first { view -> Bool in
            return view is UILabel
        }

        if let titleLabel = titleLabel {
            let titleFrame = titleLabel.convert(titleLabel.bounds, to: self)
            return max(bounds.maxY - titleFrame.maxY, 0)
        } else {
            return 0
        }
    }

    func configureCustomAppeareance() {
        var margins = layoutMargins
        margins.left = UIMetrics.contentLayoutMargins.left
        margins.right = UIMetrics.contentLayoutMargins.right

        layoutMargins = margins
        tintColor = UIColor.NavigationBar.titleColor
        backgroundColor = UIColor.NavigationBar.backgroundColor
        isTranslucent = false

        standardAppearance = makeNavigationBarAppearance()
        scrollEdgeAppearance = makeNavigationBarAppearance()
    }

    private func makeNavigationBarAppearance() -> UINavigationBarAppearance {
        let backIndicatorImage = UIImage(named: "IconBack")?.withTintColor(
            UIColor.NavigationBar.backButtonIndicatorColor,
            renderingMode: .alwaysOriginal
        )
        let backIndicatorTransitionMask = UIImage(named: "IconBackTransitionMask")

        let titleTextAttributes: [NSAttributedString.Key: Any] = [
            .foregroundColor: UIColor.NavigationBar.titleColor,
        ]
        let backButtonTitlePositionOffset = UIOffset(horizontal: 4, vertical: 0)
        let backButtonTitleTextAttributes: [NSAttributedString.Key: Any] = [
            .foregroundColor: UIColor.NavigationBar.backButtonTitleColor,
        ]

        let navigationBarAppearance = UINavigationBarAppearance()
        navigationBarAppearance.configureWithTransparentBackground()
        navigationBarAppearance.titleTextAttributes = titleTextAttributes
        navigationBarAppearance.largeTitleTextAttributes = titleTextAttributes

        let plainBarButtonAppearance = UIBarButtonItemAppearance(style: .plain)
        plainBarButtonAppearance.normal.titleTextAttributes = titleTextAttributes

        let doneBarButtonAppearance = UIBarButtonItemAppearance(style: .done)
        doneBarButtonAppearance.normal.titleTextAttributes = titleTextAttributes

        let backButtonAppearance = UIBarButtonItemAppearance(style: .plain)
        backButtonAppearance.normal.titlePositionAdjustment = backButtonTitlePositionOffset
        backButtonAppearance.normal.titleTextAttributes = backButtonTitleTextAttributes

        navigationBarAppearance.buttonAppearance = plainBarButtonAppearance
        navigationBarAppearance.doneButtonAppearance = doneBarButtonAppearance
        navigationBarAppearance.backButtonAppearance = backButtonAppearance

        if #available(iOS 14, *) {
            navigationBarAppearance.setBackIndicatorImage(
                backIndicatorImage,
                transitionMaskImage: backIndicatorTransitionMask
            )
        } else {
            // Bug: on iOS 13 setBackIndicatorImage accepts parameters in backward order
            // https://stackoverflow.com/a/58171229/351305
            navigationBarAppearance.setBackIndicatorImage(
                backIndicatorTransitionMask,
                transitionMaskImage: backIndicatorImage
            )
        }

        return navigationBarAppearance
    }
}
