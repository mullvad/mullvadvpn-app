//
//  MullvadStateView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-10.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import SwiftUI

// MARK: - Text Style
enum MullvadStateViewStyle {}

extension MullvadStateViewStyle {
    enum TextEmphasis {
        case none
        case bold
        case italic
        case boldItalic

        fileprivate var isBold: Bool {
            switch self {
            case .bold, .boldItalic:
                true
            default:
                false
            }
        }

        fileprivate var isItalic: Bool {
            switch self {
            case .italic, .boldItalic:
                true
            default:
                false
            }
        }
    }

    // MARK: - Text Style
    struct TextStyle {
        let emphasis: TextEmphasis
        let font: Font
        let color: Color
        let alignment: TextAlignment
        let padding: EdgeInsets

        init(
            emphasis: TextEmphasis = .none,
            font: Font,
            color: Color,
            alignment: TextAlignment = .leading,
            padding: EdgeInsets = EdgeInsets(
                top: 0,
                leading: 0,
                bottom: 16,
                trailing: 0
            )
        ) {
            self.emphasis = emphasis
            self.font = font
            self.color = color
            self.alignment = alignment
            self.padding = padding
        }
    }
}

extension MullvadStateViewStyle.TextStyle {
    static func headline(
        _ emphasis: MullvadStateViewStyle.TextEmphasis = .bold,
        font: Font = .mullvadLarge,
        alignment: TextAlignment = .center,
        padding: EdgeInsets = Self.defaultPadding
    ) -> Self {
        Self(
            emphasis: emphasis,
            font: font,
            color: .mullvadTextPrimary,
            alignment: alignment,
            padding: padding
        )
    }

    static func primary(
        _ emphasis: MullvadStateViewStyle.TextEmphasis = .none,
        font: Font = .mullvadSmall,
        alignment: TextAlignment = .leading,
        padding: EdgeInsets = Self.defaultPadding
    ) -> Self {
        Self(
            emphasis: emphasis,
            font: font,
            color: .mullvadTextPrimary,
            alignment: alignment,
            padding: padding
        )
    }

    static func secondary(
        _ emphasis: MullvadStateViewStyle.TextEmphasis = .none,
        font: Font = .mullvadSmall,
        alignment: TextAlignment = .leading,
        padding: EdgeInsets = Self.defaultPadding
    ) -> Self {
        Self(
            emphasis: emphasis,
            font: font,
            color: .mullvadTextSecondary,
            alignment: alignment,
            padding: padding
        )
    }

    fileprivate static var defaultPadding: EdgeInsets {
        return EdgeInsets(
            top: 0,
            leading: 0,
            bottom: 16,
            trailing: 0
        )
    }
}

// MARK: - Layout
private enum Layout {
    static let topPadding: CGFloat = 0
    static let horizontalPadding: CGFloat = 16
    static let buttonHorizontalPadding: CGFloat = 8
    static let bottomPadding: CGFloat = 24
    static let sectionSpacing: CGFloat = 24
    static let bannerSpacing: CGFloat = 16
}

// MARK: - State View Model
final class StateViewModel: Identifiable, ObservableObject {
    let id = UUID()
    @Published var style: MullvadStateView.Style
    let title: MullvadStateView.TextItem
    let banner: Image?
    let details: [MullvadStateView.TextItem]
    let explanation: MullvadStateView.TextItem?
    let actions: [MullvadStateView.ActionItem]

    init(
        style: MullvadStateView.Style,
        title: MullvadStateView.TextItem,
        banner: Image? = nil,
        details: [MullvadStateView.TextItem] = [],
        explanation: MullvadStateView.TextItem? = nil,
        actions: [MullvadStateView.ActionItem] = []
    ) {
        self.style = style
        self.title = title
        self.banner = banner
        self.details = details
        self.explanation = explanation
        self.actions = actions
    }
}

// MARK: - Main State View
struct MullvadStateView: View {
    @ObservedObject var viewModel: StateViewModel

    var body: some View {
        ScrollView {
            VStack(spacing: 0) {
                StateView(state: viewModel.style)
                    .padding(.bottom, Layout.sectionSpacing)

                StyledTextView(item: viewModel.title)

                if let banner = viewModel.banner {
                    ResizableImageView(image: banner, dimension: .width(.infinity))
                        .padding(.bottom, Layout.bannerSpacing)
                }

                ForEach(viewModel.details) { item in
                    StyledTextView(item: item)
                }

                Spacer()

                if let explanation = viewModel.explanation {
                    StyledTextView(item: explanation)
                }

                VStack(spacing: 12) {
                    ForEach(viewModel.actions) { action in
                        ActionButton(action: action)
                    }
                }
                .padding(.top, 8)
            }
            .padding(.top, Layout.topPadding)
            .padding(.horizontal, Layout.horizontalPadding)
            .padding(.bottom, Layout.bottomPadding)
        }
    }
}

extension MullvadStateView {
    struct CustomImage: Equatable {
        let id: UUID = UUID()
        let image: Image
    }

    // MARK: - State Style
    enum Style: Equatable {
        case idle
        case info
        case loading
        case error
        case success
        case custom(CustomImage)
    }

    // MARK: - Text Item
    struct TextItem: Identifiable {
        let id = UUID()
        let text: String
        var symbols: [Image] = []
        let style: MullvadStateViewStyle.TextStyle
    }

    // MARK: - Action State
    struct ActionState {
        let kind: Kind
        let message: String

        enum Kind {
            case idle, loading, success, failure

            var icon: Image? {
                switch self {
                case .idle, .loading, .failure: nil
                case .success: .mullvadIconTick
                }
            }
        }
    }

    // MARK: - Action Model
    final class ActionItem: ObservableObject, Identifiable {
        let id = UUID()
        @Published var state: ActionState
        let style: MullvadButton.Style
        var onTap: (() -> Void)

        var displayedTitle: String {
            state.message
        }

        var isLoading: Bool {
            if case .loading = state.kind {
                return true
            }
            return false
        }

        var isDisabled: Bool {
            switch state.kind {
            case .loading, .success:
                true
            default:
                false
            }
        }

        init(
            style: MullvadButton.Style,
            state: ActionState,
            onTap: @escaping (() -> Void) = {}
        ) {
            self.style = style
            self.state = state
            self.onTap = onTap
        }
    }

}

// MARK: - Action Button
private struct ActionButton: View {
    @ObservedObject var action: MullvadStateView.ActionItem
    @ScaledMetric private var baseSize: CGFloat = 24.0

    var body: some View {
        MullvadButton(
            text: "\(action.displayedTitle)",
            style: action.style,
            leadingAccessory: action.state.kind.icon.map { .icon($0) }
        ) {
            action.onTap()
        }
        .disabled(action.isDisabled)
        .padding(.horizontal, Layout.buttonHorizontalPadding)
        .overlay(alignment: .leading) {
            if action.isLoading {
                ProgressView()
                    .progressViewStyle(
                        MullvadProgressViewStyle(size: baseSize)
                    )
                    .padding(.leading, 16.0)
                    .padding(.vertical, 4.0)
            }
        }

    }
}

// MARK: - Styled Text View

private struct StyledTextView: View {
    let item: MullvadStateView.TextItem

    var body: some View {
        textContent
            .modifier(TextStyleModifier(style: item.style))
    }

    private var textContent: Text {
        buildText(
            from: item.text,
            symbols: item.symbols
        )
    }

    private func buildText(
        from template: String,
        symbols: [Image]
    ) -> Text {

        let parts = template.components(separatedBy: "%@")

        return parts.enumerated().reduce(Text("")) { result, pair in
            let (index, part) = pair
            let symbolText: Text = {
                guard index < symbols.count else {
                    return Text("")
                }

                return Text(
                    "\(symbols[index].renderingMode(.template))"
                )
            }()

            return result + Text(part) + symbolText
        }
    }
}

private struct TextStyleModifier: ViewModifier {
    let style: MullvadStateViewStyle.TextStyle

    func body(content: Content) -> some View {
        content
            .font(style.font)
            .foregroundStyle(style.color)
            .multilineTextAlignment(style.alignment)
            .frame(
                maxWidth: .infinity,
                alignment: style.alignment.frameAlignment
            )
            .padding(style.padding)
            .modifier(EmphasisModifier(style: style))
    }
}

private struct EmphasisModifier: ViewModifier {
    let style: MullvadStateViewStyle.TextStyle

    func body(content: Content) -> some View {
        switch style {
        case let style where style.emphasis.isBold && style.emphasis.isItalic:
            content
                .bold()
                .italic()

        case let style where style.emphasis.isBold:
            content.bold()

        case let style where style.emphasis.isItalic:
            content.italic()

        default:
            content
        }
    }
}

// MARK: - Alignment Helpers
extension TextAlignment {
    fileprivate var frameAlignment: Alignment {
        switch self {
        case .leading:
            return .leading
        case .center:
            return .center
        case .trailing:
            return .trailing
        }
    }
}

// MARK: - State Icon View
private struct StateView: View {
    let state: MullvadStateView.Style

    private let size = 48.0

    var body: some View {
        content
            .animation(.default, value: state)
    }

    @ViewBuilder
    private var content: some View {
        switch state {
        case .idle:
            EmptyView()

        case .loading:
            ProgressView()
                .progressViewStyle(MullvadProgressViewStyle(size: size))

        case .success:
            ResizableImageView(
                image: Image.mullvadIconSuccess,
                dimension: .width(size)
            )

        case .error:
            ResizableImageView(
                image: Image.mullvadIconError,
                dimension: .width(size)
            )

        case .info:
            ResizableImageView(
                image: Image.mullvadIconInfo,
                dimension: .width(size)
            )
        case .custom(let customImage):
            ResizableImageView(image: customImage.image, dimension: .width(size))
        }
    }
}

#Preview() {
    MullvadStateViewPreviewWrapper()
}
