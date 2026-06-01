//
//  MullvadStateView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-10.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import SwiftUI

// MARK: - State Style
enum StateViewStyle {
    case idle
    case info
    case loading
    case error
    case success
}

// MARK: - Text Style
enum TextEmphasis {
    case none
    case bold
    case italic
    case boldItalic
}

enum TextStyle {
    case headline(TextEmphasis, EdgeInsets)
    case primary(TextEmphasis, EdgeInsets)
    case secondary(TextEmphasis, EdgeInsets)

    static func headline(
        _ emphasis: TextEmphasis = .none,
        padding: EdgeInsets = Self.defaultPadding
    ) -> Self {
        .headline(emphasis, padding)
    }

    static func primary(
        _ emphasis: TextEmphasis = .none,
        padding: EdgeInsets = Self.defaultPadding
    ) -> Self {
        .primary(emphasis, padding)
    }

    static func secondary(
        _ emphasis: TextEmphasis = .none,
        padding: EdgeInsets = Self.defaultPadding
    ) -> Self {
        .secondary(emphasis, padding)
    }
}

extension TextStyle {
    private var emphasis: TextEmphasis {
        switch self {
        case .headline(let emphasis, _),
            .primary(let emphasis, _),
            .secondary(let emphasis, _):
            return emphasis
        }
    }

    var isBold: Bool {
        switch emphasis {
        case .bold, .boldItalic:
            return true

        case .none, .italic:
            return false
        }
    }

    var isItalic: Bool {
        switch emphasis {
        case .italic, .boldItalic:
            return true

        case .none, .bold:
            return false
        }
    }

    var font: Font {
        switch self {
        case .headline:
            return .mullvadLarge

        case .primary,
            .secondary:
            return .mullvadSmall
        }
    }

    var color: Color {
        switch self {
        case .headline,
            .primary:
            return .mullvadTextPrimary

        case .secondary:
            return .mullvadTextSecondary
        }
    }

    var alignment: TextAlignment {
        switch self {
        case .headline:
            return .center

        case .primary,
            .secondary:
            return .leading
        }
    }

    var padding: EdgeInsets {
        switch self {
        case .headline(_, let padding),
            .primary(_, let padding),
            .secondary(_, let padding):
            return padding
        }
    }

    static var defaultPadding: EdgeInsets {
        return EdgeInsets(
            top: 0,
            leading: 0,
            bottom: 16,
            trailing: 0
        )
    }
}

// MARK: - Layout
enum Layout {
    static let topPadding: CGFloat = 16
    static let horizontalPadding: CGFloat = 16
    static let buttonHorizontalPadding: CGFloat = 8
    static let bottomPadding: CGFloat = 24
    static let sectionSpacing: CGFloat = 24
    static let bannerSpacing: CGFloat = 16
}

// MARK: - Text Item
struct TextItem: Identifiable {
    let id = UUID()
    let text: String
    var symbols: [Image] = []
    let style: TextStyle
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
    let style: MainButtonStyle.Style
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
        style: MainButtonStyle.Style,
        state: ActionState,
        onTap: @escaping (() -> Void) = {}
    ) {
        self.style = style
        self.state = state
        self.onTap = onTap
    }
}

// MARK: - State View Model
final class StateViewModel: Identifiable, ObservableObject {
    let id = UUID()
    @Published var style: StateViewStyle
    let title: TextItem
    let banner: Image?
    let details: [TextItem]
    let explanation: TextItem?
    let actions: [ActionItem]

    init(
        style: StateViewStyle,
        title: TextItem,
        banner: Image? = nil,
        details: [TextItem],
        explanation: TextItem? = nil,
        actions: [ActionItem] = []
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
        GeometryReader { geo in
            ScrollView {
                VStack(spacing: 0) {
                    StateView(state: viewModel.style)
                        .padding(.bottom, Layout.sectionSpacing)

                    StyledTextView(item: viewModel.title)

                    if let banner = viewModel.banner {
                        ResizableImageView(image: banner, layout: .banner)
                            .padding(.bottom, Layout.bannerSpacing)
                    }

                    ForEach(viewModel.details) { item in
                        StyledTextView(item: item)
                    }

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
                .frame(
                    maxWidth: .infinity,
                    minHeight: geo.size.height,
                    alignment: .top
                )
            }
        }
    }
}

// MARK: - Action Button
struct ActionButton: View {
    @ObservedObject var action: ActionItem
    @ScaledMetric private var baseSize: CGFloat = 24.0

    var body: some View {
        MainButton(
            text: "\(action.displayedTitle)",
            style: action.style
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
            } else if let icon = action.state.kind.icon {
                ResizableImageView(image: icon, layout: .square(baseSize))
                    .padding(.leading, 16.0)
                    .foregroundStyle(Color.mullvadTextPrimary)
                    .padding(.vertical, 4.0)
            }
        }

    }
}

// MARK: - Styled Text View

struct StyledTextView: View {
    let item: TextItem

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

struct TextStyleModifier: ViewModifier {
    let style: TextStyle

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

struct EmphasisModifier: ViewModifier {
    let style: TextStyle

    func body(content: Content) -> some View {
        switch style {
        case let style where style.isBold && style.isItalic:
            content
                .bold()
                .italic()

        case let style where style.isBold:
            content.bold()

        case let style where style.isItalic:
            content.italic()

        default:
            content
        }
    }
}

// MARK: - Alignment Helpers
extension TextAlignment {
    var frameAlignment: Alignment {
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
struct StateView: View {
    let state: StateViewStyle

    @ScaledMetric private var size = 48

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
                layout: .square(size)
            )

        case .error:
            ResizableImageView(
                image: Image.mullvadIconError,
                layout: .square(size)
            )

        case .info:
            ResizableImageView(
                image: Image.mullvadIconInfo,
                layout: .square(size)
            )
        }
    }
}

#Preview() {
    MullvadStateViewPreviewWrapper()
}
