import SwiftUI

/// A plain simple list.
/// * separators reaching all the way
/// * the height of all items in the list are based on the highest element
/// * the list is not higher than all its items
/// * transparent background
struct MullvadList<Content: View, Data: RandomAccessCollection<ID>, ID: Hashable>: View {
    let data: Data
    let content: (Data.Element) -> Content
    let id: KeyPath<Data.Element, ID>?

    @State var itemHeight: CGFloat = 0
    var maxListHeight: CGFloat {
        let height = itemHeight * CGFloat(data.count)
        return height > 0 ? height : .infinity
    }

    init(_ data: Data, id: KeyPath<Data.Element, ID>, @ViewBuilder content: @escaping (Data.Element) -> Content) {
        self.data = data
        self.id = id
        self.content = content
    }

    init(_ data: Data, @ViewBuilder content: @escaping (Data.Element) -> Content) {
        self.data = data
        self.content = content
        self.id = nil
    }

    var body: some View {
        List(data, id: id ?? \.self) { item in
            content(item)
                .sizeOfView { size in
                    if itemHeight < size.height {
                        itemHeight = size.height
                    }
                }
                .listRowInsets(.init())
                .listSectionSeparator(.hidden, edges: .bottom)
                .listRowSeparatorTint(.MullvadList.separator)
                .listRowBackground(Color.clear)
                .apply {
                    if #available(iOS 16.0, *) {
                        $0.alignmentGuide(.listRowSeparatorLeading) { _ in
                            0
                        }
                    } else {
                        $0
                    }
                }
                .frame(height: itemHeight)
        }
        .listStyle(.plain)
        .frame(maxHeight: maxListHeight)
    }
}

#Preview {
    MullvadList([1, 2, 3]) { item in
        VStack {
            ForEach(0 ..< item, id: \.self) {
                Text("\($0)")
            }
        }
        .padding()
    }
}
