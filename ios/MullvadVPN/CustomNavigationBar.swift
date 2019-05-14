//
//  CustomNavigationBar.swift
//  MullvadVPN
//
//  Created by pronebird on 03/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import UIKit

class CustomNavigationBar: UINavigationBar {
    private(set) var isBarVisible = false

    private let emptyShadow = UIImage()

    /// The blur view used internally by UINavigationBar
    private var effectView: UIVisualEffectView? {
        // Find the background view in the navigation bar view hierarchy
        let backgroundView = subviews.first(where: { $0.description.starts(with: "<_UIBarBackground") })

        // Find the blur view in the background view's view hierarchy
        let backgroundEffectView = backgroundView?.subviews.first(where: { $0 is UIVisualEffectView })

        return backgroundEffectView as? UIVisualEffectView
    }

    /// The custom title view or the standard title label used internally by UINavigationBar
    private var titleView: UIView? {
        // Return the custom title view when it's set
        if let customTitleView = topItem?.titleView {
            return customTitleView
        }

        // Find the content view inside of the navigation bar hierarchy
        let contentView = subviews.first(where: { $0.description.starts(with: "<_UINavigationBarContentView") })

        // Find the UILabel in the content view's subviews
        return contentView?.subviews.first(where: { $0 is UILabel })
    }

    override func layoutSubviews() {
        super.layoutSubviews()

        // UINavigationBar creates subviews dynamically, so make sure to reset the navigation bar state
        setBarBackgroundVisibility(isBarVisible)

        // UINavigationBar tends to reset the title view opacity in response to layout changes
        setTitleVisibility(isBarVisible)
    }

    func setBarVisible(_ visible: Bool, animated: Bool) {
        guard isBarVisible != visible else { return }

        isBarVisible = visible

        let action = {
            self.setBarBackgroundVisibility(visible)
            self.setTitleVisibility(visible)
        }

        if animated {
            UIView.animate(withDuration: 0.25, delay: 0,
                           options: [.beginFromCurrentState],
                           animations: action)
        } else {
            action()
        }
    }

    private func setBarBackgroundVisibility(_ visible: Bool) {
        let backgroundEffectView = effectView

        if visible {
            backgroundEffectView?.alpha = 1
            shadowImage = nil
        } else {
            backgroundEffectView?.alpha = 0
            shadowImage = emptyShadow
        }
    }

    private func setTitleVisibility(_ visible: Bool) {
        titleView?.alpha = visible ? 1 : 0
    }
}
