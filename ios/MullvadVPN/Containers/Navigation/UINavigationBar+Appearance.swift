//
//  CustomNavigationBar.swift
//  MullvadVPN
//
//  Created by pronebird on 22/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UINavigationBar {
    func configureCustomAppeareance() {
        var directionalMargins = directionalLayoutMargins
        directionalMargins.leading = UIMetrics.contentDirectionalLayoutMargins.leading
        directionalMargins.trailing = UIMetrics.contentDirectionalLayoutMargins.trailing

        directionalLayoutMargins = directionalMargins
        tintColor = UIColor.NavigationBar.titleColor

        standardAppearance = makeNavigationBarAppearance(isTransparent: false)
        scrollEdgeAppearance = makeNavigationBarAppearance(isTransparent: true)
    }

    private func makeNavigationBarAppearance(isTransparent: Bool) -> UINavigationBarAppearance {
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

        if isTransparent {
            navigationBarAppearance.configureWithTransparentBackground()
        } else {
            navigationBarAppearance.configureWithDefaultBackground()
            navigationBarAppearance.backgroundEffect = UIBlurEffect(style: .dark)
        }

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
