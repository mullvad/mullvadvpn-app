// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

#if NEVER_IN_PRODUCTION

    import UIKit

    extension SceneDelegate {
        func setUpLogOverlay() {
            guard let windowScene = window?.windowScene else { return }

            let interactor = InAppLogViewInteractor(observer: appDelegate.inAppLogObserver)
            let viewController = InAppLogOverlayViewController(interactor: interactor)

            let logWindow = PassthroughWindow(windowScene: windowScene)
            logWindow.windowLevel = .statusBar + 1
            logWindow.backgroundColor = .clear
            logWindow.rootViewController = viewController

            self.logWindow = logWindow

            let tapGesture = UITapGestureRecognizer(target: self, action: #selector(logOverlayTapSequenceActivated))
            tapGesture.delegate = self
            tapGesture.numberOfTapsRequired = 2
            tapGesture.cancelsTouchesInView = false
            tapGesture.delaysTouchesEnded = false

            self.window?.addGestureRecognizer(tapGesture)
        }

        @objc private func logOverlayTapSequenceActivated() {
            logWindow?.isHidden.toggle()
        }
    }

    // Pass taps on to regular controls so as not to block them.
    extension SceneDelegate: UIGestureRecognizerDelegate {
        func gestureRecognizer(_ gestureRecognizer: UIGestureRecognizer, shouldReceive touch: UITouch) -> Bool {
            true
        }
    }

    private class PassthroughWindow: UIWindow {
        override func hitTest(_ point: CGPoint, with event: UIEvent?) -> UIView? {
            let hit = super.hitTest(point, with: event)
            return hit === self ? nil : hit
        }
    }

#endif
