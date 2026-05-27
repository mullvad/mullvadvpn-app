import * as grpcTypesRelaySelector from 'management-interface/relay-selector/grpc-types';

import {
  RelaySelectorPartitions,
  RelaySelectorPredicate,
  RelaySelectorPredicateEntryConstraints,
  RelaySelectorPredicateGeneralConstraints,
  RelaySelectorPredicateMultihopConstraints,
  RelaySelectorProvider,
  RelaySelectorRelayDiscard,
  RelaySelectorRelayDiscardWhy,
  RelaySelectorRelayMatch,
} from '../shared/relay-selector-rpc-types';
import {
  convertToAntiCensorshipSettings,
  convertToDaitaSettings,
  convertToIpVersion,
  convertToLocation,
  convertToOwnership,
  unwrapConstraint,
} from './grpc-type-convertions';

export function convertToRelaySelectorProvider(
  provider: RelaySelectorProvider,
): grpcTypesRelaySelector.Provider {
  const grpcProvider = new grpcTypesRelaySelector.Provider();

  grpcProvider.setName(provider.name);

  return grpcProvider;
}

export function convertToRelaySelectorProviders(
  providers: RelaySelectorProvider[],
): grpcTypesRelaySelector.Provider[] {
  return providers.map((provider) => convertToRelaySelectorProvider(provider));
}

export function convertToRelaySelectorGeneralConstraints(
  generalConstraints: RelaySelectorPredicateGeneralConstraints,
): grpcTypesRelaySelector.ExitConstraints {
  const grpcGeneralConstraints = new grpcTypesRelaySelector.ExitConstraints();

  const location = unwrapConstraint(generalConstraints.location);
  if (location) {
    grpcGeneralConstraints.setLocation(convertToLocation(location));
  }

  grpcGeneralConstraints.setProvidersList(
    convertToRelaySelectorProviders(generalConstraints.providers),
  );
  grpcGeneralConstraints.setOwnership(convertToOwnership(generalConstraints.ownership));

  return grpcGeneralConstraints;
}

export function convertToRelaySelectorEntryConstraints(
  constraints: RelaySelectorPredicateEntryConstraints,
): grpcTypesRelaySelector.EntryConstraints {
  const grpcEntryConstraints = new grpcTypesRelaySelector.EntryConstraints();
  const generalConstraints = convertToRelaySelectorGeneralConstraints(
    constraints.generalConstraints,
  );
  grpcEntryConstraints.setGeneralConstraints(generalConstraints);

  if (constraints.antiCensorship) {
    const grpcAntiCensorshipSettings = convertToAntiCensorshipSettings(constraints.antiCensorship);

    // TODO: Rename gRPC names from Obfuscation to AntiCensorship
    grpcEntryConstraints.setObfuscationSettings(grpcAntiCensorshipSettings);
  }

  if (constraints.daitaSettings) {
    const grpcDaitaSettings = convertToDaitaSettings(constraints.daitaSettings);
    grpcEntryConstraints.setDaitaSettings(grpcDaitaSettings);
  }

  if (constraints.ipVersion) {
    const unconstrainedIpVersion = unwrapConstraint(constraints.ipVersion);
    if (unconstrainedIpVersion) {
      const grpcIpVersion = convertToIpVersion(unconstrainedIpVersion);
      grpcEntryConstraints.setIpVersion(grpcIpVersion);
    }
  }

  return grpcEntryConstraints;
}

export function convertToRelaySelectorMultihopConstraints(
  predicate: RelaySelectorPredicateMultihopConstraints,
): grpcTypesRelaySelector.MultiHopConstraints {
  const grpcMultihopConstraints = new grpcTypesRelaySelector.MultiHopConstraints();

  const entry = convertToRelaySelectorEntryConstraints(predicate.entry);
  grpcMultihopConstraints.setEntry(entry);

  const exit = convertToRelaySelectorGeneralConstraints(predicate.exit);
  grpcMultihopConstraints.setExit(exit);

  return grpcMultihopConstraints;
}

export function convertToRelaySelectorPredicate(
  predicate: RelaySelectorPredicate,
): grpcTypesRelaySelector.Predicate {
  const grpcPredicate = new grpcTypesRelaySelector.Predicate();

  if ('singlehop' in predicate) {
    const grpcEntryConstraints = convertToRelaySelectorEntryConstraints(predicate.singlehop);
    grpcPredicate.setSinglehop(grpcEntryConstraints);

    return grpcPredicate;
  }

  if ('autohop' in predicate) {
    const grpcEntryConstraints = convertToRelaySelectorEntryConstraints(predicate.autohop);
    grpcPredicate.setAutohop(grpcEntryConstraints);

    return grpcPredicate;
  }

  if ('entry' in predicate) {
    const grpcMultihopConstraints = convertToRelaySelectorMultihopConstraints(predicate.entry);
    grpcPredicate.setEntry(grpcMultihopConstraints);

    return grpcPredicate;
  }

  if ('exit' in predicate) {
    const grpcMultihopConstraints = convertToRelaySelectorMultihopConstraints(predicate.exit);
    grpcPredicate.setExit(grpcMultihopConstraints);

    return grpcPredicate;
  }

  return predicate satisfies unknown;
}

export function convertFromRelaySelectorRelay(
  relayMatch: grpcTypesRelaySelector.Relay,
): RelaySelectorRelayMatch {
  return {
    hostname: relayMatch.getHostname(),
  };
}

export function convertFromRelaySelectorRelayMatch(
  relayMatch: grpcTypesRelaySelector.Relay,
): RelaySelectorRelayMatch {
  return convertFromRelaySelectorRelay(relayMatch);
}

export function convertFromRelaySelectorRelayMatchesList(
  relayMatchesList: grpcTypesRelaySelector.Relay[],
): RelaySelectorRelayMatch[] {
  return relayMatchesList.map((relayMatch) => convertFromRelaySelectorRelayMatch(relayMatch));
}

export function convertFromRelaySelectorDiscardWhy(
  why: grpcTypesRelaySelector.IncompatibleConstraints,
): RelaySelectorRelayDiscardWhy {
  return {
    antiCensorship: why.getObfuscation(), // TODO: We should rename all instances of "obfuscation" in the proto file with the "anti-censorship" name
    conflictWithOtherHop: why.getConflictWithOtherHop(),
    daita: why.getDaita(),
    inactive: why.getInactive(),
    ipVersion: why.getIpVersion(),
    location: why.getLocation(),
    ownership: why.getOwnership(),
    port: why.getPort(),
    providers: why.getProviders(),
  };
}

export function convertFromRelaySelectorDiscard(
  relayDiscard: grpcTypesRelaySelector.DiscardedRelay,
): RelaySelectorRelayDiscard {
  const relay = convertFromRelaySelectorRelay(relayDiscard.getRelay()!); // TODO: Update the proto file to ensure this is a required field
  const why = convertFromRelaySelectorDiscardWhy(relayDiscard.getWhy()!); // TODO: Update the proto file to ensure this is a required field

  return {
    relay,
    why,
  };
}

export function convertFromRelaySelectorDiscardsList(
  relayDiscardsList: grpcTypesRelaySelector.DiscardedRelay[],
): RelaySelectorRelayDiscard[] {
  return relayDiscardsList.map((relayDiscard) => convertFromRelaySelectorDiscard(relayDiscard));
}

export function convertFromRelaySelectorPartitions(
  relayPartitions: grpcTypesRelaySelector.RelayPartitions,
): RelaySelectorPartitions {
  const matches = convertFromRelaySelectorRelayMatchesList(relayPartitions.getMatchesList());
  const discards = convertFromRelaySelectorDiscardsList(relayPartitions.getDiscardsList());

  return {
    matches,
    discards,
  };
}
