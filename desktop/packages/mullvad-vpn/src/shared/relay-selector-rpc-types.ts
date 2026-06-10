import {
  Constraint,
  IpVersion,
  ObfuscationSettings,
  Ownership,
  RelayLocationGeographical,
} from './daemon-rpc-types';

export type RelaySelectorRelayDiscard = {
  relay: RelaySelectorRelayMatch;
  why: RelaySelectorRelayDiscardWhy;
};

export type RelaySelectorRelayDiscardWhy = {
  // The relay is currently offline.
  inactive: boolean;
  // The relay does not reside in the given location.
  location: boolean;
  // The relay is not hosted by the given providers.
  providers: boolean;
  // The relay ownership does not match.
  ownership: boolean;
  // -- Entry specific constraints --
  // The relay cannot be connected to with the requested ip version.
  ipVersion: boolean;
  // The relay does not run DAITA.
  daita: boolean;
  // The requested obfuscation method is not available.
  antiCensorship: boolean;
  // The relay cannot be connected to via the requested port.
  port: boolean;
  // This relay is already used for the other hop (entry/exit).
  conflictWithOtherHop: boolean;
};

export type RelaySelectorRelay = {
  hostname: string;
};

export type RelaySelectorRelayMatch = RelaySelectorRelay;

export type RelaySelectorPartitions = {
  matches: RelaySelectorRelayMatch[];
  discards: RelaySelectorRelayDiscard[];
};

export type RelaySelectorProvider = {
  name: string;
};

export type RelaySelectorPredicateGeneralConstraints = {
  location: Constraint<RelayLocationGeographical>;
  providers: RelaySelectorProvider[];
  ownership: Ownership;
};

export type RelaySelectorPredicateEntryConstraints = {
  generalConstraints: RelaySelectorPredicateGeneralConstraints;
  antiCensorship: ObfuscationSettings;
  daita: boolean;
  ipVersion: Constraint<IpVersion>;
};

export type RelaySelectorPredicateMultihopConstraints = {
  entry: RelaySelectorPredicateEntryConstraints;
  exit: RelaySelectorPredicateGeneralConstraints;
};

export type RelaySelectorPredicate =
  | {
      type: 'singlehop';
      constraints: RelaySelectorPredicateEntryConstraints;
    }
  | {
      type: 'autohop';
      constraints: RelaySelectorPredicateEntryConstraints;
    }
  | {
      type: 'multihopEntry';
      constraints: RelaySelectorPredicateMultihopConstraints;
    }
  | {
      type: 'multihopExit';
      constraints: RelaySelectorPredicateMultihopConstraints;
    };
