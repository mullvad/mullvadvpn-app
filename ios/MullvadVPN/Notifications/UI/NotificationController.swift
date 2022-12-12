//
//  NotificationController.swift
//  MullvadVPN
//
//  Created by pronebird on 01/06/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

final class NotificationController: UIViewController {
    let bannerView: NotificationBannerView = {
        let bannerView = NotificationBannerView()
        bannerView.translatesAutoresizingMaskIntoConstraints = false
        bannerView.isHidden = true
        bannerView.isAccessibilityElement = true
        return bannerView
    }()

    private var showBannerConstraint: NSLayoutConstraint?
    private var hideBannerConstraint: NSLayoutConstraint?

    private(set) var showsBanner = false
    private var lastNotification: InAppNotificationDescriptor?

    override func loadView() {
        view = NotificationContainerView(frame: UIScreen.main.bounds)
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        showBannerConstraint = bannerView.topAnchor.constraint(equalTo: view.topAnchor)
        hideBannerConstraint = bannerView.bottomAnchor.constraint(equalTo: view.topAnchor)

        view.addSubview(bannerView)

        let verticalConstraint = showsBanner ? showBannerConstraint : hideBannerConstraint
        NSLayoutConstraint.activate([
            verticalConstraint!,
            bannerView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            bannerView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
        ])
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()

        updateAccessibilityFrame()
    }

    func toggleBanner(show: Bool, animated: Bool, completion: (() -> Void)? = nil) {
        guard showsBanner != show else {
            completion?()
            return
        }

        showsBanner = show

        if show {
            // Make sure to lay out the banner before animating its appearance to
            // avoid undesired horizontal expansion animation.
            view.layoutIfNeeded()

            bannerView.isHidden = false
            hideBannerConstraint?.isActive = false
            showBannerConstraint?.isActive = true
        } else {
            showBannerConstraint?.isActive = false
            hideBannerConstraint?.isActive = true
        }

        let finish = {
            if !show {
                self.bannerView.isHidden = true
            }
            completion?()
        }

        if animated {
            let timing = UISpringTimingParameters(
                dampingRatio: 0.7,
                initialVelocity: CGVector(dx: 0, dy: 1)
            )
            let animator = UIViewPropertyAnimator(duration: 0.8, timingParameters: timing)
            animator.isInterruptible = false
            animator.addAnimations {
                self.view.layoutIfNeeded()
            }
            animator.addCompletion { _ in
                finish()
            }
            animator.startAnimation()
        } else {
            view.layoutIfNeeded()
            finish()
        }
    }

    func setNotification(_ notification: InAppNotificationDescriptor, animated: Bool) {
        guard lastNotification != notification else { return }

        lastNotification = notification

        bannerView.title = notification.title
        bannerView.body = notification.body
        bannerView.style = notification.style
        bannerView.accessibilityLabel = "\(notification.title)\n\(notification.body)"

        if animated {
            let animator = UIViewPropertyAnimator(
                duration: 0.25,
                timingParameters: UICubicTimingParameters(animationCurve: .easeOut)
            )
            animator.addAnimations {
                self.view.layoutIfNeeded()
            }
            animator.startAnimation()
        }

        // Do not emit the .layoutChanged unless the banner is focused to avoid capturing
        // the voice over focus.
        if bannerView.accessibilityElementIsFocused() {
            UIAccessibility.post(notification: .layoutChanged, argument: bannerView)
        }
    }

    func setNotifications(_ notifications: [InAppNotificationDescriptor], animated: Bool) {
        let nextNotification = notifications.first

        if let notification = nextNotification {
            setNotification(notification, animated: showsBanner)
            toggleBanner(show: true, animated: true)
        } else {
            toggleBanner(show: false, animated: animated)
        }
    }

    private func updateAccessibilityFrame() {
        let layoutFrame = bannerView.layoutMarginsGuide.layoutFrame
        bannerView.accessibilityFrame = UIAccessibility.convertToScreenCoordinates(
            layoutFrame,
            in: view
        )
    }
}
