import SwiftUI

struct EntryLocationView<ViewModel: SelectLocationViewModel>: View {
    @ObservedObject var viewModel: ViewModel
    let isSearching: Bool
    let onScrollOffsetChange: (CGFloat, CGFloat) -> Void
    var body: some View {
        if viewModel.showDAITAInfo {
            DaitaWarningView {
                viewModel.showDaitaSettings()
            }
        } else {
            ExitLocationView(
                viewModel: viewModel, context: $viewModel.entryContext,
                isSearching: isSearching, onScrollOffsetChange: onScrollOffsetChange)
        }
    }
}
