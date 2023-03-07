//
//  PresentationControllerDismissalInterceptor.swift
//  MullvadVPN
//
//  Created by pronebird on 20/02/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/**
 Presentation controller delegate class that intercepts interactive dismissal and calls
 `dismissHandler` closure. Forwards all delegate calls to the `forwardingTarget`.
 */
final class PresentationControllerDismissalInterceptor: NSObject,
    UIAdaptivePresentationControllerDelegate
{
    private let dismissHandler: (UIPresentationController) -> Void
    private let forwardingTarget: UIAdaptivePresentationControllerDelegate?
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
        return super.responds(to: aSelector) || (
            protocolSelectors.contains(aSelector) &&
                forwardingTarget?.responds(to: aSelector) ?? false
        )
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

    return (0 ..< methodCount).compactMap { index in
        return methodDescriptions?[Int(index)].name
    }
}
