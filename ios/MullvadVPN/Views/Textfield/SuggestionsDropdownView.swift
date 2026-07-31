//
//  SuggestionsDropdownView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-07-24.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import SwiftUI

struct SuggestionsDropdownView: View {
    var font: Font = .mullvadSmall
    var foregroundColor: Color = .mullvadTextPrimary
    var dividerColor: Color = .mullvadTextSecondary
    var borderStyle: BorderStyle = .none
    var cornerRadius: CGFloat = 4.0
    var backgroundColor: Color = Color(red: 38.0 / 255.0, green: 71.0 / 255.0, blue: 106.0 / 255.0)

    private var suggestions: [String] = []
    let onSelect: (String) -> Void
    let onRemove: ((String) -> Void)?

    init(
        suggestions: [String],
        onSelect: @escaping (String) -> Void,
        onRemove: ((String) -> Void)? = nil
    ) {
        self.suggestions = suggestions
        self.onSelect = onSelect
        self.onRemove = onRemove
    }

    private var suggestionsHeight: CGFloat {
        let dividerHeight = max(0, suggestions.count - 1)
        let totalHeight =
            CGFloat(suggestions.count) * SuggestionsDropdownUIMetrics.minHeight + CGFloat(dividerHeight)

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
                                    minHeight: SuggestionsDropdownUIMetrics.minHeight,
                                    alignment: .leading
                                )
                                .font(font)
                                .foregroundStyle(foregroundColor)
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

                    }
                    .buttonStyle(.plain)

                    if index < suggestions.count - 1 {
                        Divider()
                            .background(dividerColor)
                    }
                }
            }
        }
        .frame(height: suggestionsHeight)
        .background(backgroundColor)
        .modifier(
            RoundedCornerModifier(
                cornerRadius: cornerRadius,
                corners: [.bottomLeft, .bottomRight],
                insertBy: .zero,
                borderColor: borderStyle.color,
                borderWidth: borderStyle.lineWidth
            )
        )
    }
}

private enum SuggestionsDropdownUIMetrics {
    static let minHeight: CGFloat = 44.0
}

extension SuggestionsDropdownView {
    func font(_ font: Font) -> Self {
        var copy = self
        copy.font = font
        return copy
    }

    func foregroundColor(_ color: Color) -> Self {
        var copy = self
        copy.foregroundColor = color
        return copy
    }

    func borderStyle(_ style: BorderStyle) -> Self {
        var copy = self
        copy.borderStyle = style
        return copy
    }

    func dividerColor(_ color: Color) -> Self {
        var copy = self
        copy.dividerColor = color
        return copy
    }

    func cornerRadius(_ radius: CGFloat) -> Self {
        var copy = self
        copy.cornerRadius = radius
        return copy
    }

    func backgroundColor(_ color: Color) -> Self {
        var copy = self
        copy.backgroundColor = color
        return copy
    }
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
