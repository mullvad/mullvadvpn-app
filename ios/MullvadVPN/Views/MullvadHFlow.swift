import SwiftUI

struct MullvadHFlow<Data, Content>: View where Data: RandomAccessCollection, Content: View, Data.Element: Hashable, Data.Element: Sendable {
    let spacing: CGFloat
    let alignment: HorizontalAlignment
    let items: Data
//    let id: KeyPath<Data.Element, ID>

    @State private var contentSize: [ItemSize] = []

    struct ItemSize: Sendable {
        let item: Data.Element
        let size: CGSize
    }

    struct PositionInFlow {
        let item: Data.Element
        let xOffset: CGFloat
        let row: Int
    }

    func calculatePositions(contentWidth: CGFloat) -> [PositionInFlow] {
        var positions: [PositionInFlow] = []
        var currentX: CGFloat = 0
        var currentRow = 0

        // swiftlint:disable:next unused_enumerated
        for item in items {
            guard let itemSize = contentSize.first(where: { $0.item == item })?.size else {
                positions.append(PositionInFlow(item: item, xOffset: 0, row: 0))
                continue
            }
            if currentX + itemSize.width + spacing > contentWidth {
                currentX = 0
                currentRow += 1
            }
            let position = PositionInFlow(
                item: item,
                xOffset: currentX,
                row: currentRow
            )
            positions.append(position)
            currentX += itemSize.width + spacing
        }
        return positions
    }

    @ViewBuilder let content: (Data.Element) -> Content
    @State var viewHeight: CGFloat = 0
    init(
        spacing: CGFloat = 8,
        alignment: HorizontalAlignment = .leading,
        _ items: Data,
        @ViewBuilder content: @escaping (Data.Element) -> Content
    ) where Content: View {
        self.spacing = spacing
        self.alignment = alignment
        self.content = content
        self.items = items
    }

    var body: some View {
        GeometryReader { geo in
            ZStack(alignment: .topLeading) {
                let positions = calculatePositions(contentWidth: geo.size.width)
                ForEach(items, id: \.self) { item in
                    let safeContentSize = contentSize
                    if let position = positions.first(
                        where: { $0.item == item
                        }) {
                        content(item)
                            .sizeOfView { size in
                                let currSizeIndex = contentSize.firstIndex { $0.item == item }
                                if let currSizeIndex {
                                    self.contentSize.remove(at: currSizeIndex)
                                }
                                self.contentSize.append(.init(item: item, size: size))
                            }
                            .alignmentGuide(.leading) { _ in
                                return -position.xOffset
                            }
                            .alignmentGuide(.top) { _ in
                                let maxElementHeight = safeContentSize.map { $0.size.height }.max() ?? 0
                                return -CGFloat(position.row) * (maxElementHeight + spacing)
                            }
                    }
                }
            }
            .sizeOfView { size in
                withAnimation {
                    self.viewHeight = size.height
                }
            }
        }
        .frame(height: viewHeight)
    }
}

@available(iOS 17.0, *)
#Preview {
    @Previewable @State var items = ["Test", "TestTest"]
    VStack(alignment: .leading) {
        HStack(alignment: .top) {
            Text("Bla")
            MullvadHFlow(spacing: 8, items) { item in
                VStack {
                    if item.count > 4 {
                        Text(item)
                    }
                    Text(item)
                }
                .background(
                    Color(hue: .random(in: 0 ... 1), saturation: 1, brightness: 1)
                )
            }
            .background(Color.yellow)
        }
        Button("Add element") {
            print(items)
            items.append(String(repeating: "42", count: Int.random(in: 1 ..< 5)))
        }
        Button("Remove element") {
            items.remove(at: .random(in: 0 ..< items.count))
        }
    }
}
