import SwiftUI

struct ActiveFilterView: View {
    let activeFilter: [SelectLocationFilter]
    let onSelect: (SelectLocationFilter) -> Void
    let onRemove: (SelectLocationFilter) -> Void
    @State private var maxItemHeight: CGFloat = 0
    var body: some View {
        HStack(alignment: .top) {
            Text("Filtered:")
                .font(.mullvadTiny)
            MullvadHFlow(activeFilter) { filter in
                Button {
                    onSelect(filter)
                } label: {
                    HStack {
                        Text(filter.title)
                            .font(.mullvadMiniSemiBold)
                            .foregroundStyle(Color.mullvadTextPrimary)
                        if filter.canBeRemoved {
                            Button {
                                onRemove(filter)
                            } label: {
                                Image.mullvadIconCross
                            }
                        }
                    }
                    .padding(8)
                    .sizeOfView { size in
                        maxItemHeight = max(maxItemHeight, size.height)
                    }
                    .frame(height: maxItemHeight)
                    .background {
                        RoundedRectangle(cornerRadius: 8)
                            .foregroundStyle(Color.MullvadButton.primary)
                    }
                }
            }
        }
    }
}

#Preview {
    Text("da")
        .sheet(isPresented: .constant(true)) {
            NavigationView {
                ScrollView {
                    ActiveFilterView(
                        activeFilter: [.daita, .owned, .rented, .provider(2)],
                        onSelect: { _ in
                        },
                        onRemove: { _ in }
                    )
                }
            }
        }
}
