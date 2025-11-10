//
//  View+Modifier.swift
//  MullvadVPN
//
//  Created by Steffen Ernst on 2025-01-21.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

extension View {
    /**
      A view modifier that can be used to conditionally apply other view modifiers.
     */
    func apply<V: View>(@ViewBuilder _ block: (Self) -> V) -> V { block(self) }

    /**
     Uses the AccessibilityIdentifier you specify to identify the view.
      # Discussion #
     Use this value for testing. It isn’t visible to the user.
     */
    func accessibilityIdentifier(_ id: AccessibilityIdentifier?) -> some View {
        apply {
            if let id {
                $0.accessibilityIdentifier(id.asString)
            } else {
                $0
            }
        }
    }
}
