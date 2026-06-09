import {
  Constraint,
  IpVersion,
  IRelayList,
  IRelayListCity,
  IRelayListCountry,
  IRelayListHostname,
  ObfuscationType,
  Ownership,
  RelayLocationGeographical,
} from '../../../src/shared/daemon-rpc-types';
import {
  RelaySelectorPartitions,
  RelaySelectorPredicate,
  RelaySelectorPredicateEntryConstraints,
  RelaySelectorProvider,
  RelaySelectorRelayDiscardWhy,
} from '../../../src/shared/relay-selector-rpc-types';

function getDiscardCausedByAntiCensorship(
  relay: IRelayListHostname,
  selectedObfuscation: ObfuscationType,
) {
  switch (selectedObfuscation) {
    case ObfuscationType.quic ?? false:
      return relay.quic ? false : true;
    case ObfuscationType.lwo:
      return !relay.lwo;
    default:
      return false;
  }
}

function getDiscardCausedByIpVersion(relay: IRelayListHostname, ipVersion: Constraint<IpVersion>) {
  if (ipVersion !== 'any') {
    if (ipVersion.only === 'ipv4' && !relay.ipv4AddrIn) {
      return true;
    }

    if (ipVersion.only === 'ipv6' && !relay.ipv6AddrIn) {
      return true;
    }
  }

  return false;
}

function getDiscardCausedByLocation(
  relayLocation: Constraint<RelayLocationGeographical> | undefined,
  relay: IRelayListHostname,
  city: IRelayListCity,
  country: IRelayListCountry,
) {
  if (relayLocation && relayLocation !== 'any') {
    if ('hostname' in relayLocation.only) {
      return relay.hostname !== relayLocation.only.hostname;
    }

    if ('city' in relayLocation.only) {
      return city.code !== relayLocation.only.city;
    }

    if ('country' in relayLocation.only) {
      return country.code !== relayLocation.only.country;
    }
  }

  return false;
}

function getDiscardCausedByOwnership(ownership: Ownership | undefined, relay: IRelayListHostname) {
  switch (ownership) {
    case Ownership.mullvadOwned:
      return !relay.owned;
    case Ownership.rented:
      return !!relay.owned;
    default:
      return false;
  }
}

function getDiscardCausedByProviders(
  providers: RelaySelectorProvider[],
  relay: IRelayListHostname,
) {
  if (providers.length > 0) {
    return !providers.some(({ name }) => name === relay.provider);
  }

  return false;
}

function getDiscardCausedByConflictWithOtherHop(
  relayLocation: Constraint<RelayLocationGeographical> | undefined,
  relay: IRelayListHostname,
) {
  if (relayLocation && relayLocation !== 'any') {
    // Only exact hostname will cause the conflictWithOtherHop discard reason to be triggered
    if ('hostname' in relayLocation.only) {
      return relay.hostname === relayLocation.only.hostname;
    }
  }

  return false;
}

function getDiscardCausedByEntryConstraints(
  constraints: RelaySelectorPredicateEntryConstraints,
  relay: IRelayListHostname,
  city: IRelayListCity,
  country: IRelayListCountry,
) {
  const discardReasons = {
    antiCensorship: false,
    conflictWithOtherHop: false,
    daita: false,
    inactive: false,
    ipVersion: false,
    location: false,
    ownership: false,
    port: false,
    providers: false,
  };

  discardReasons.antiCensorship = getDiscardCausedByAntiCensorship(
    relay,
    constraints.antiCensorship.selectedObfuscation,
  );
  discardReasons.daita = !relay.daita;
  discardReasons.inactive = !relay.active;
  discardReasons.ipVersion = getDiscardCausedByIpVersion(relay, constraints.ipVersion);
  discardReasons.location = getDiscardCausedByLocation(
    constraints.generalConstraints.location,
    relay,
    city,
    country,
  );
  discardReasons.ownership = getDiscardCausedByOwnership(
    constraints.generalConstraints.ownership,
    relay,
  );
  // NOTE: Mock behavior to discard a relay based on port selection is currently unimplemented
  // due to no test existing which uses this feature.
  //
  // If the needs arises then support can be added here by comparing against the `port_ranges` in
  // the wireguard endpoint data.
  discardReasons.port = false;
  discardReasons.providers = getDiscardCausedByProviders(
    constraints.generalConstraints.providers,
    relay,
  );

  return discardReasons;
}

function getRelayDiscardReasons(
  { type, constraints }: RelaySelectorPredicate,
  relay: IRelayListHostname,
  city: IRelayListCity,
  country: IRelayListCountry,
): RelaySelectorRelayDiscardWhy {
  const discardReasons = {
    antiCensorship: false,
    conflictWithOtherHop: false,
    daita: false,
    inactive: false,
    ipVersion: false,
    location: false,
    ownership: false,
    port: false,
    providers: false,
  };

  if (type === 'multihopEntry') {
    const entryDiscardReasons = getDiscardCausedByEntryConstraints(
      constraints.entry,
      relay,
      city,
      country,
    );
    Object.assign(discardReasons, entryDiscardReasons);

    discardReasons.conflictWithOtherHop = getDiscardCausedByConflictWithOtherHop(
      constraints.exit.location,
      relay,
    );
  }

  if (type === 'multihopExit') {
    discardReasons.location = getDiscardCausedByLocation(
      constraints.exit.location,
      relay,
      city,
      country,
    );
    discardReasons.providers = getDiscardCausedByProviders(constraints.exit.providers, relay);
    discardReasons.conflictWithOtherHop = getDiscardCausedByConflictWithOtherHop(
      constraints.entry.generalConstraints.location,
      relay,
    );
  } else if (type === 'singlehop' || type === 'autohop') {
    const entryDiscardReasons = getDiscardCausedByEntryConstraints(
      constraints,
      relay,
      city,
      country,
    );
    Object.assign(discardReasons, entryDiscardReasons);
  }

  return discardReasons;
}

export function getRelayPartitions(
  predicate: RelaySelectorPredicate,
  relayList: IRelayList,
): RelaySelectorPartitions {
  const relayDiscardReasonsPartitions = relayList.countries.flatMap((country) =>
    country.cities.flatMap((city) =>
      city.relays.flatMap((relay) => ({
        hostname: relay.hostname,
        discardReasons: getRelayDiscardReasons(predicate, relay, city, country),
      })),
    ),
  );

  const relayPartitions = relayDiscardReasonsPartitions.reduce(
    (partitions, { hostname, discardReasons }) => {
      const discarded = Object.values(discardReasons).some((discardReason) => discardReason);

      if (discarded) {
        return {
          ...partitions,
          discards: [...partitions.discards, { relay: { hostname }, why: discardReasons }],
        };
      }

      return {
        ...partitions,
        matches: [...partitions.matches, { hostname }],
      };
    },
    {
      discards: [],
      matches: [],
    } as RelaySelectorPartitions,
  );

  return relayPartitions;
}
