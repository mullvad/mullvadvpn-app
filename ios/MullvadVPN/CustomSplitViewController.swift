//
//  CustomSplitViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 07/04/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

class CustomSplitViewController: UISplitViewController, RootContainment {

    var preferredHeaderBarStyle: HeaderBarStyle {
        for case let viewController as RootContainment in viewControllers {
            return viewController.preferredHeaderBarStyle
        }
        return .default
    }

    var prefersHeaderBarHidden: Bool {
        for case let viewController as RootContainment in viewControllers {
            return viewController.prefersHeaderBarHidden
        }
        return false
    }

    var dividerColor: UIColor? {
        didSet {
            if isViewLoaded {
                self.updateDividerColor()
            }
        }
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()

        updateDividerColor()
    }

    private var dividerView: UIView? {
        let subviews = view.subviews.flatMap { (view) -> [UIView] in
            return [view] + view.subviews
        }

        return subviews.first { (view) -> Bool in
            return view.description.hasPrefix("<UIPanelBorderView")
        }
    }

    private func updateDividerColor() {
        guard let dividerColor = dividerColor else { return }

        dividerView?.backgroundColor = dividerColor
    }

}
