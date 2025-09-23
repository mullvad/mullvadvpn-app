import SwiftUI

/// A plain simple list.
/// * separators reaching all the way
/// * the height of all items in the list are based on the highest element
/// * the list is not higher than all its items
/// * transparent background
struct MullvadList<Content: View, Data: RandomAccessCollection<ID>, ID: Hashable>: View {
    let data: Data
    let id: KeyPath<Data.Element, ID>
    let content: (Data.Element) -> Content
    let header: (() -> AnyView)?
    let footer: (() -> AnyView)?

    init(
        _ data: Data,
        id: KeyPath<Data.Element, ID>,
        header: (() -> some View)? = nil,
        footer: (() -> some View)? = nil,
        @ViewBuilder content: @escaping (Data.Element) -> Content
    ) {
        self.data = data
        self.id = id
        self.header = header.map { builder in { AnyView(builder()) } }
        self.footer = footer.map { builder in { AnyView(builder()) } }
        self.content = content
    }

    init(
        _ data: Data,
        header: (() -> some View)? = nil,
        footer: (() -> some View)? = nil,
        @ViewBuilder content: @escaping (Data.Element) -> Content
    ) where Data.Element == ID {
        self.init(data, id: \.self, header: header, footer: footer, content: content)
    }

    var body: some View {
        VStack(alignment: .leading) {
            List {
                if let headerView = header?() {
                    headerView
                        .listRowStyling(insets: EdgeInsets(UIMetrics.contentHeadingLayoutMargins))
                }

                let lastItem = data.last
                ForEach(data, id: id) { item in
                    VStack(spacing: 0) {
                        content(item)
                        if item != lastItem {
                            RowSeparator()
                        }
                    }
                    .listRowStyling()
                }

                if let footerView = footer?() {
                    footerView
                        .listRowStyling(insets: EdgeInsets(UIMetrics.contentFooterLayoutMargins))
                }
            }
            .listStyle(.plain)
            .listRowSpacing(.zero)
            .environment(\.defaultMinListRowHeight, 0)
        }
    }
}

extension View {
    func listRowStyling(
        background: Color = .clear,
        separator: Visibility = .hidden,
        insets: EdgeInsets = .init()
    ) -> some View {
        apply {
            $0
                .listRowBackground(background)
                .listRowSeparator(separator)
                .listRowInsets(insets)
        }
    }
}

#Preview {
    MullvadList(
        [1, 2, 3],
        header: {
            Text("Header")
        },
        footer: {
            Text("Footer")
        },
        content: { item in
            Text("Item \(item)").padding()
        }
    )
}
