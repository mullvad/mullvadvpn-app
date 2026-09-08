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

// MARK: - Layout
private enum Layout {
    static let topPadding: CGFloat = 0
    static let horizontalPadding: CGFloat = 16
    static let buttonHorizontalPadding: CGFloat = 16
    static let bottomPadding: CGFloat = 24
    static let sectionSpacing: CGFloat = 24
    static let bannerSpacing: CGFloat = 16
}

// MARK: - State View Model
final class NoticeViewModel: Identifiable, ObservableObject {
    let id = UUID()
    @Published var style: MullvadNoticeView.Style
    let title: MullvadNoticeView.TextItem
    let banner: Image?
    let details: [MullvadNoticeView.TextItem]
    let explanation: MullvadNoticeView.TextItem?
    let actions: [MullvadNoticeView.ActionItem]

    init(
        style: MullvadNoticeView.Style,
        title: MullvadNoticeView.TextItem,
        banner: Image? = nil,
        details: [MullvadNoticeView.TextItem] = [],
        explanation: MullvadNoticeView.TextItem? = nil,
        actions: [MullvadNoticeView.ActionItem] = []
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
struct MullvadNoticeView: View {
    @ObservedObject var viewModel: NoticeViewModel
    @State private var actionHeight: CGFloat = 0

    var body: some View {
        ZStack {
            ScrollView {
                ZStack {
                    Spacer().containerRelativeFrame([.horizontal, .vertical])
                    VStack(spacing: 0) {
                        Spacer()
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
                    }
                    .padding(.top, Layout.topPadding)
                    .padding(.horizontal, Layout.horizontalPadding)
                    .padding(.bottom, actionHeight)
                }
            }
            VStack(spacing: 12) {
                Spacer()
                Group {
                    ForEach(viewModel.actions) { action in
                        ActionButton(action: action)
                    }
                }
                .padding(.bottom, Layout.bottomPadding)
                .sizeOfView { size in
                    self.actionHeight = size.height
                }
            }
        }
    }
}

extension MullvadNoticeView {
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
        let style: MullvadNoticeView.Style.Text
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

// MARK: - Text Style

extension MullvadNoticeView.Style {
    // MARK: - Text Style
    struct Text {
        enum Emphasis {
            case none
            case bold
            case italic
            case boldItalic
        }

        let emphasis: Emphasis
        let font: Font
        let color: Color
        let alignment: TextAlignment
        let padding: EdgeInsets

        init(
            emphasis: Emphasis = .none,
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

extension MullvadNoticeView.Style.Text {
    static func headline(
        _ emphasis: MullvadNoticeView.Style.Text.Emphasis = .bold,
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
        _ emphasis: MullvadNoticeView.Style.Text.Emphasis = .none,
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
        _ emphasis: MullvadNoticeView.Style.Text.Emphasis = .none,
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

extension MullvadNoticeView.Style.Text.Emphasis {
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

// MARK: - Action Button
private struct ActionButton: View {
    @ObservedObject var action: MullvadNoticeView.ActionItem
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
    let item: MullvadNoticeView.TextItem

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
    let style: MullvadNoticeView.Style.Text

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
    let style: MullvadNoticeView.Style.Text

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
    let state: MullvadNoticeView.Style

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
    MullvadNoticeViewPreviewWrapper()
}
