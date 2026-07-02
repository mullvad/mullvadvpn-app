//
//  ScrollView+Position.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-06-26.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct ViewOffsetKey: PreferenceKey {
    static var defaultValue: CGFloat { 0 }

    static func reduce(value: inout CGFloat, nextValue: () -> CGFloat) {
        value = nextValue()
    }
}

struct ScrollVisibilityModifier: ViewModifier {
    let coordinateSpace: AnyHashable
    let threshold: CGFloat
    let onChange: (Bool) -> Void

    @State private var lastValue = false

    func body(content: Content) -> some View {
        content
            .background {
                GeometryReader { proxy in
                    Color.clear.preference(
                        key: ViewOffsetKey.self,
                        value: proxy.frame(in: .named(coordinateSpace)).minY
                    )
                }
            }
            .onPreferenceChange(ViewOffsetKey.self) { position in
                let visible = position >= threshold
                guard visible != lastValue else { return }

                lastValue = visible
                onChange(visible)
            }
    }
}

extension View {
    func scrollVisibility(
        in coordinateSpace: AnyHashable,
        threshold: CGFloat = 40,
        onChange: @escaping (Bool) -> Void
    ) -> some View {
        modifier(
            ScrollVisibilityModifier(
                coordinateSpace: coordinateSpace,
                threshold: threshold,
                onChange: onChange
            )
        )
    }
}
