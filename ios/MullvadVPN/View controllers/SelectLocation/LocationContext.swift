struct LocationContext {
    var locations: [LocationNode]
    var customLists: [LocationNode]
    var filter: [SelectLocationFilter]
    let selectLocation: (LocationNode) -> Void

    init(
        locations: [LocationNode] = [],
        customLists: [LocationNode] = [],
        filter: [SelectLocationFilter] = [],
        selectedLocation: LocationNode? = nil,
        connectedRelayHostname: String? = nil,
        selectLocation: @escaping (LocationNode) -> Void = { _ in }
    ) {
        self.locations = locations
        self.customLists = customLists
        self.filter = filter
        self.selectLocation = selectLocation
    }

    var selectedLocation: LocationNode? {
        (locations + customLists)
            .flatMap { [$0] + $0.flattened }
            .first { $0.isSelected }
    }
}
