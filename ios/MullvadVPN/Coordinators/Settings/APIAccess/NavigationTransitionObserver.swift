// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import SwiftUI
import UIKit

final class NavigationTransitionObserver: NSObject, UINavigationControllerDelegate {
    var onTransitionCompleted: (() -> Void)?

    func navigationController(
        _ navigationController: UINavigationController,
        didShow viewController: UIViewController,
        animated: Bool
    ) {
        onTransitionCompleted?()
    }
}

struct NavigationDelegateAccessor: UIViewControllerRepresentable {
    let observer: NavigationTransitionObserver

    func makeUIViewController(context: Context) -> UIViewController {
        let controller = UIViewController()
        controller.view.isHidden = true
        return controller
    }

    func updateUIViewController(
        _ controller: UIViewController,
        context: Context
    ) {
        DispatchQueue.main.async {
            guard
                let navigationController =
                    controller.navigationController
            else {
                return
            }

            navigationController.delegate = observer
        }
    }
}
