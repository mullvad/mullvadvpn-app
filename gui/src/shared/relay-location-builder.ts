import { Constraint, LiftedConstraint, RelayLocation } from './daemon-rpc-types';

export interface ILocationBuilder<Self> {
  country(country: string): Self;
  city(country: string, city: string): Self;
  hostname(country: string, city: string, hostname: string): Self;
  any(): Self;
  fromRaw(location: LiftedConstraint<RelayLocation>): Self;
}

export default function makeLocationBuilder<T>(
  context: T,
  receiver: (constraint: Constraint<RelayLocation>) => void,
): ILocationBuilder<T> {
  return {
    country: (country: string) => {
      receiver({ only: { country } });
      return context;
    },
    city: (country: string, city: string) => {
      receiver({ only: { country, city } });
      return context;
    },
    hostname: (country: string, city: string, hostname: string) => {
      receiver({ only: { country, city, hostname } });
      return context;
    },
    any: () => {
      receiver('any');
      return context;
    },
    fromRaw(location: LiftedConstraint<RelayLocation>) {
      if (location === 'any') {
        return this.any();
      } else {
        receiver({ only: location });
        return context;
      }
    },
  };
}
