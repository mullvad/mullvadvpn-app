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

struct SuggestionsDropdownView: View {
    private var suggestions: [String] = []
    let appearance: SuggestionsDropdownViewAppearance
    let onSelect: (String) -> Void
    let onRemove: ((String) -> Void)?

    init(
        suggestions: [String],
        appearance: SuggestionsDropdownViewAppearance = .init(),
        onSelect: @escaping (String) -> Void,
        onRemove: ((String) -> Void)? = nil
    ) {
        self.suggestions = suggestions
        self.appearance = appearance
        self.onSelect = onSelect
        self.onRemove = onRemove
    }

    private var suggestionsHeight: CGFloat {
        let dividerHeight = max(0, suggestions.count - 1)
        let totalHeight =
            CGFloat(suggestions.count) * appearance.height + CGFloat(dividerHeight)

        return min(totalHeight, 200)
    }

    var body: some View {
        ScrollView {
            LazyVStack(spacing: 0) {
                ForEach(Array(suggestions.enumerated()), id: \.element) { index, suggestion in
                    Button {
                        onSelect(suggestion)
                    } label: {

                        HStack {
                            Text(suggestion)
                                .frame(
                                    maxWidth: .infinity,
                                    alignment: .leading
                                )
                                .font(appearance.font)
                                .foregroundStyle(appearance.foregroundColor)
                                .padding(.horizontal, 8.0)

                            if let onRemove = onRemove {
                                Button(action: {
                                    onRemove(suggestion)
                                }) {
                                    ResizableImageView(image: .mullvadIconCross, dimension: .width(24.0))
                                }
                                .buttonStyle(.plain)
                                .padding(.trailing, 8)
                            }

                        }
                        .frame(minHeight: appearance.height)
                        .contentShape(Rectangle())

                    }
                    .buttonStyle(.plain)

                    if index < suggestions.count - 1 {
                        Divider()
                            .background(appearance.dividerColor)
                    }
                }
            }
        }
        .frame(height: suggestionsHeight)
        .background(appearance.backgroundColor)
        .scrollBounceBehavior(.basedOnSize)
        .modifier(
            RoundedCornerModifier(
                cornerRadius: appearance.cornerRadius,
                corners: [.bottomLeft, .bottomRight],
                insertBy: .zero,
                borderColor: appearance.borderStyle.color,
                borderWidth: appearance.borderStyle.lineWidth
            )
        )
    }
}

struct SuggestionsDropdownViewAppearance {
    var font: Font = .mullvadSmall
    var foregroundColor: Color = .MullvadSuggestionsDropdown.foregroundColor
    var dividerColor: Color = .MullvadSuggestionsDropdown.separator
    var borderStyle: BorderStyle = .normal
    var cornerRadius: CGFloat = 4.0
    var backgroundColor: Color = .MullvadSuggestionsDropdown.background
    var height: CGFloat = 44.0
}

#Preview {
    SuggestionsDropdownView(
        suggestions: [
            "Apple",
            "Apricot",
            "Avocado",
        ],
        onSelect: { item in
            print("Selected: \(item)")
        }
    )
    .padding()
}
