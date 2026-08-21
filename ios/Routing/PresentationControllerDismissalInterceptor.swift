// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import UIKit

/**
 Presentation controller delegate class that intercepts interactive dismissal and calls
 `dismissHandler` closure. Forwards all delegate calls to the `forwardingTarget`.
 */
final class PresentationControllerDismissalInterceptor: NSObject,
    UIAdaptivePresentationControllerDelegate
{
    private let dismissHandler: (UIPresentationController) -> Void
    nonisolated(unsafe) private let forwardingTarget: UIAdaptivePresentationControllerDelegate?
    private let protocolSelectors: [Selector]

    init(
        forwardingTarget: UIAdaptivePresentationControllerDelegate?,
        dismissHandler: @escaping (UIPresentationController) -> Void
    ) {
        self.forwardingTarget = forwardingTarget
        self.dismissHandler = dismissHandler

        protocolSelectors = getProtocolMethods(
            UIAdaptivePresentationControllerDelegate.self,
            isRequired: false,
            isInstanceMethod: true
        )
    }

    override func responds(to aSelector: Selector!) -> Bool {
        super.responds(to: aSelector)
            || (protocolSelectors.contains(aSelector) && forwardingTarget?.responds(to: aSelector) ?? false)
    }

    override func forwardingTarget(for aSelector: Selector!) -> Any? {
        if protocolSelectors.contains(aSelector) {
            if super.responds(to: aSelector) {
                return nil
            } else if forwardingTarget?.responds(to: aSelector) ?? false {
                return forwardingTarget
            }
        }
        return super.forwardingTarget(for: aSelector)
    }

    func presentationControllerDidDismiss(_ presentationController: UIPresentationController) {
        dismissHandler(presentationController)
        forwardingTarget?.presentationControllerDidDismiss?(presentationController)
    }
}

private func getProtocolMethods(
    _ protocolType: Protocol,
    isRequired: Bool,
    isInstanceMethod: Bool
) -> [Selector] {
    var methodCount: UInt32 = 0
    let methodDescriptions = protocol_copyMethodDescriptionList(
        protocolType,
        isRequired,
        isInstanceMethod,
        &methodCount
    )

    defer { methodDescriptions.map { free($0) } }

    return (0..<methodCount).compactMap { index in
        methodDescriptions?[Int(index)].name
    }
}
