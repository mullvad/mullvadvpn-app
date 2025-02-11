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
      # Example #
     ```
     .apply {
        if #available(iOS 16.4, *) {
            $0.scrollBounceBehavior(.basedOnSize)
        } else {
            $0
        }
     }
     ```
     */
    func apply<V: View>(@ViewBuilder _ block: (Self) -> V) -> V { block(self) }
}
