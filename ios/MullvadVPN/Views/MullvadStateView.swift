//
//  MullvadStateView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-10.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

enum StateViewStyle {
    case idle
    case info
    case loading
    case error
    case success
}

final class StateViewModel: ObservableObject {
    @Published var style: StateViewStyle = .idle
    @Published var title: TextItem = .init(text: "", style: .headline)
    @Published var banner: Image? = nil
    @Published var details: [TextItem] = []
    @Published var explanation: TextItem? = nil
    @Published var actions: [Action]? = nil

    init(
        style: StateViewStyle,
        title: TextItem, banner: Image? = nil,
        details: [TextItem],
        explanation: TextItem? = nil,
        actions: [Action]? = nil
    ) {
        self.style = style
        self.title = title
        self.banner = banner
        self.details = details
        self.explanation = explanation
        self.actions = actions
    }
}

struct TextStyle {
    let font: Font
    let color: Color
    let alignment: TextAlignment
    var isItalic: Bool = false

    static var headline: TextStyle {
        TextStyle(
            font: .mullvadLarge,
            color: .mullvadTextPrimary,
            alignment: .center)
    }

    static var primary: TextStyle {
        TextStyle(
            font: .mullvadSmall,
            color: .mullvadTextPrimary,
            alignment: .leading)
    }

    static var secondary: TextStyle {
        TextStyle(
            font: .mullvadSmall,
            color: .mullvadTextSecondary,
            alignment: .leading)
    }

    static var note: TextStyle {
        TextStyle(
            font: .mullvadSmall,
            color: .mullvadTextSecondary,
            alignment: .leading,
            isItalic: true)
    }
}

struct TextItem: Identifiable {
    let id = UUID()
    let text: String
    var symbols: [Image] = []
    let style: TextStyle
}

// MARK: - Action Model

struct Action: Identifiable {
    let id = UUID()
    let title: String
    let style: MainButtonStyle.Style
    let onTap: (() -> Void)
}

// MARK: - StateView

struct MullvadStateView: View {
    @ObservedObject var viewModel: StateViewModel

    init(viewModel: StateViewModel) {
        self.viewModel = viewModel
    }

    var body: some View {
        GeometryReader { geo in
            ScrollView {
                VStack(spacing: 16.0) {
                    Spacer()
                    StateView(state: viewModel.style)
                        .padding(.bottom, 8.0)

                    StyledTextView(item: viewModel.title)

                    if let banner = viewModel.banner {
                        ResizableImageView(image: banner, layout: .banner)
                    }

                    ForEach(viewModel.details, id: \.id) { item in
                        StyledTextView(item: item)
                    }

                    Spacer()

                    if let explanation = viewModel.explanation {
                        StyledTextView(item: explanation)
                    }

                    ForEach(viewModel.actions ?? [], id: \.id) { action in
                        MainButton(text: LocalizedStringKey(action.title), style: action.style) {
                            action.onTap()
                        }
                        .padding(.horizontal, 8.0)
                    }
                    Spacer()
                }
                .padding(.horizontal, 16)
                .padding(.bottom, 24)
                .frame(maxWidth: .infinity)
                .frame(minHeight: geo.size.height)
            }
        }
    }
}

struct StyledTextView: View {
    let item: TextItem
    private var alignment: Alignment {
        switch item.style.alignment {
        case .leading: return .leading
        case .trailing: return .trailing
        case .center: return .center
        }
    }

    var body: some View {
        buildText(from: item.text, symbols: item.symbols)
            .font(item.style.font)
            .multilineTextAlignment(item.style.alignment)
            .foregroundStyle(item.style.color)
            .frame(maxWidth: .infinity, alignment: alignment)
            .if(item.style.isItalic) { view in
                view.italic()
            }
    }

    private func buildText(from template: String, symbols: [Image]) -> Text {
        let parts = template.components(separatedBy: "%@")

        return parts.enumerated().reduce(Text("")) { partial, pair in
            let (index, part) = pair

            var result = partial + Text(part)

            if index < symbols.count {
                result = result + iconText(symbols[index])
            }

            return result
        }
    }

    private func iconText(_ image: Image) -> Text {
        Text("\(image.renderingMode(.template))")
            .font(item.style.font)
    }
}

struct StateView: View {
    @State var state: StateViewStyle = .idle
    @ScaledMetric private var size = 48

    var body: some View {
        content.animation(.default, value: state)
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
            ResizableImageView(image: Image.mullvadIconSuccess, layout: .square(size))
        case .error:
            ResizableImageView(image: Image.mullvadIconError, layout: .square(size))
        case .info:
            ResizableImageView(image: Image.mullvadIconInfo, layout: .square(size))
        }
    }
}

#Preview {
    MullvadStateView(
        viewModel: StateViewModel(
            style: .info,
            title: TextItem(text: NSLocalizedString("Suggested action", comment: ""), style: .headline),
            banner: Image(.ianSolutionIllustration),
            details: [
                TextItem(
                    text: NSLocalizedString(
                        """
                        To ensure your current settings work with your selected location, and to avoid blocking your 
                        connection, the app might automatically multihop via a different entry server.
                        This will be indicated by the  %@  symbol.
                        """, comment: ""),
                    symbols: [Image.mullvadIconMultihopWhenNeeded],
                    style: .secondary)
            ],
            actions: [
                Action(
                    title: NSLocalizedString("Change to “When needed”", comment: ""),
                    style: .default,
                    onTap: {
                        print("Tapped")
                    })
            ])
    )
    .background(Color.mullvadBackground)
}
