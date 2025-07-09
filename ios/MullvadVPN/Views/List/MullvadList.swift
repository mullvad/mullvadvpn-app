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
        List {
            if let headerView = header?() {
                headerView
                    .listRowBackground(Color.clear)
            }

            ForEach(data, id: id) { item in
                content(item)
                    .listRowInsets(.init())
                    .listSectionSeparator(.hidden, edges: .bottom)
                    .listRowSeparatorTint(.MullvadList.separator)
                    .listRowBackground(Color.clear)
            }

            if let footerView = footer?() {
                footerView
                    .listRowBackground(Color.clear)
            }
        }
        .listStyle(.plain)
    }
}

#Preview {
    MullvadList(
        [1, 2, 3],
        header: {
            Text("Header")
        }, footer: {
            Text("Footer")
        },
        content: { item in
            Text("Item \(item)").padding()
        }
    )
}
