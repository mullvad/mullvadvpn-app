//
//  CustomNavigationBar.swift
//  MullvadVPN
//
//  Created by pronebird on 22/05/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import UIKit

class CustomNavigationBar: UINavigationBar {

    private static let setupAppearanceForIOS12Once: Void = {
        if #available(iOS 13, *) {
            // no-op
        } else {
            let buttonAppearance = UIBarButtonItem.appearance(whenContainedInInstancesOf: [CustomNavigationBar.self])
            buttonAppearance.setBackButtonTitlePositionAdjustment(UIOffset(horizontal: 4, vertical: 0), for: .default)
        }
    }()

    private let customBackIndicatorImage = UIImage(named: "IconBack")
    private let customBackIndicatorTransitionMask = UIImage(named: "IconBackTransitionMask")

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
        Self.setupAppearanceForIOS12Once

        super.init(frame: frame)

        var margins = layoutMargins
        margins.left = UIMetrics.contentLayoutMargins.left
        margins.right = UIMetrics.contentLayoutMargins.right
        layoutMargins = margins

        setupNavigationBarAppearance()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func setupNavigationBarAppearance() {
        backgroundColor = .secondaryColor
        isTranslucent = false

        if #available(iOS 13, *) {
            standardAppearance = makeNavigationBarAppearance()
            scrollEdgeAppearance = makeNavigationBarAppearance()
        } else {
            backIndicatorImage = customBackIndicatorImage
            backIndicatorTransitionMaskImage = customBackIndicatorTransitionMask
            barTintColor = .secondaryColor

            let titleAttributes: [NSAttributedString.Key: Any] = [.foregroundColor: UIColor.white]
            titleTextAttributes = titleAttributes
            largeTitleTextAttributes = titleAttributes
            shadowImage = UIImage()
        }
    }

    @available(iOS 13, *)
    private func makeNavigationBarAppearance() -> UINavigationBarAppearance {
        let textAttributes: [NSAttributedString.Key: Any] = [.foregroundColor: UIColor.white]

        let navigationBarAppearance = UINavigationBarAppearance()
        navigationBarAppearance.configureWithTransparentBackground()
        navigationBarAppearance.titleTextAttributes = textAttributes
        navigationBarAppearance.largeTitleTextAttributes = textAttributes

        let plainBarButtonAppearance = UIBarButtonItemAppearance(style: .plain)
        plainBarButtonAppearance.normal.titleTextAttributes = textAttributes

        let doneBarButtonAppearance = UIBarButtonItemAppearance(style: .done)
        doneBarButtonAppearance.normal.titleTextAttributes = textAttributes

        let backButtonAppearance = UIBarButtonItemAppearance(style: .plain)
        backButtonAppearance.normal.titleTextAttributes = textAttributes
        backButtonAppearance.normal.titlePositionAdjustment = UIOffset(horizontal: 4, vertical: 0)

        navigationBarAppearance.buttonAppearance = plainBarButtonAppearance
        navigationBarAppearance.doneButtonAppearance = doneBarButtonAppearance
        navigationBarAppearance.backButtonAppearance = backButtonAppearance

        if #available(iOS 14, *) {
            navigationBarAppearance.setBackIndicatorImage(customBackIndicatorImage, transitionMaskImage: customBackIndicatorTransitionMask)
        } else {
            // Bug: on iOS 13 setBackIndicatorImage accepts parameters in backward order
            // https://stackoverflow.com/a/58171229/351305
            navigationBarAppearance.setBackIndicatorImage(customBackIndicatorTransitionMask, transitionMaskImage: customBackIndicatorImage)
        }

        return navigationBarAppearance
    }

}
