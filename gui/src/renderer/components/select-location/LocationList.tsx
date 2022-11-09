import * as React from 'react';

import { compareRelayLocation, RelayLocation } from '../../../shared/daemon-rpc-types';
import { RelayLocations } from './RelayLocations';
import { SpecialLocations } from './SpecialLocations';

export enum LocationSelectionType {
  relay = 'relay',
  special = 'special',
}

export type LocationSelection<SpecialValueType> =
  | { type: LocationSelectionType.special; value: SpecialValueType }
  | { type: LocationSelectionType.relay; value: RelayLocation };

interface ILocationListState<SpecialValueType> {
  selectedValue?: LocationSelection<SpecialValueType>;
  expandedLocations: RelayLocation[];
}

interface ILocationListProps<SpecialValueType> {
  defaultExpandedLocations?: RelayLocation[];
  selectedValue?: LocationSelection<SpecialValueType>;
  selectedElementRef?: React.Ref<React.ReactInstance>;
  onSelect?: (value: LocationSelection<SpecialValueType>) => void;
  children?: React.ReactNode;
}

export default class LocationList<SpecialValueType> extends React.Component<
  ILocationListProps<SpecialValueType>,
  ILocationListState<SpecialValueType>
> {
  public state: ILocationListState<SpecialValueType> = {
    expandedLocations: [],
  };

  public selectedRelayLocationRef: React.ReactInstance | null = null;
  public selectedSpecialLocationRef: React.ReactInstance | null = null;

  constructor(props: ILocationListProps<SpecialValueType>) {
    super(props);

    if (props.selectedValue) {
      const expandedLocations =
        props.defaultExpandedLocations ||
        (props.selectedValue.type === LocationSelectionType.relay
          ? expandRelayLocation(props.selectedValue.value)
          : []);

      this.state = {
        selectedValue: props.selectedValue,
        expandedLocations,
      };
    }
  }

  public getExpandedLocations(): RelayLocation[] {
    return this.state.expandedLocations;
  }

  public componentDidUpdate(prevProps: ILocationListProps<SpecialValueType>) {
    if (!compareLocationSelectionLoose(prevProps.selectedValue, this.props.selectedValue)) {
      this.setState({ selectedValue: this.props.selectedValue });
    }
  }

  public render() {
    const selection = this.state.selectedValue;
    const specialSelection =
      selection && selection.type === LocationSelectionType.special ? selection.value : undefined;
    const relaySelection =
      selection && selection.type === LocationSelectionType.relay ? selection.value : undefined;

    return (
      <>
        {React.Children.map(this.props.children, (child) => {
          if (React.isValidElement(child)) {
            if (child.type === SpecialLocations) {
              return React.cloneElement(child, {
                ...child.props,
                selectedElementRef: this.onSpecialLocationRef,
                selectedValue: specialSelection,
                onSelect: this.onSelectSpecialLocation,
              });
            } else if (child.type === RelayLocations) {
              return React.cloneElement(child, {
                ...child.props,
                selectedLocation: relaySelection,
                selectedElementRef: this.onRelayLocationRef,
                expandedItems: this.state.expandedLocations,
                onSelect: this.onSelectRelayLocation,
                onExpand: this.onExpandRelayLocation,
              });
            }
          }
          return child;
        })}
      </>
    );
  }

  private onSpecialLocationRef = (ref: React.ReactInstance | null) => {
    this.selectedSpecialLocationRef = ref;

    this.updateExternalRef();
  };

  private onRelayLocationRef = (ref: React.ReactInstance | null) => {
    this.selectedRelayLocationRef = ref;

    this.updateExternalRef();
  };

  private updateExternalRef() {
    if (this.props.selectedElementRef) {
      const value = this.selectedRelayLocationRef || this.selectedSpecialLocationRef;

      if (typeof this.props.selectedElementRef === 'function') {
        this.props.selectedElementRef(value);
      } else {
        const ref = this.props
          .selectedElementRef as React.MutableRefObject<React.ReactInstance | null>;
        ref.current = value;
      }
    }
  }

  private onSelectRelayLocation = (value: RelayLocation) => {
    const selectedValue: LocationSelection<SpecialValueType> = {
      type: LocationSelectionType.relay,
      value,
    };

    this.setState({ selectedValue }, () => {
      this.notifySelection(selectedValue);
    });
  };

  private onSelectSpecialLocation = (value: SpecialValueType) => {
    const selectedValue: LocationSelection<SpecialValueType> = {
      type: LocationSelectionType.special,
      value,
    };

    this.setState({ selectedValue }, () => {
      this.notifySelection(selectedValue);
    });
  };

  private notifySelection(value: LocationSelection<SpecialValueType>) {
    if (this.props.onSelect) {
      this.props.onSelect(value);
    }
  }

  private onExpandRelayLocation = (location: RelayLocation, expand: boolean) => {
    this.setState((state) => {
      const expandedLocations = state.expandedLocations.filter(
        (item) => !compareRelayLocation(item, location),
      );

      if (expand) {
        expandedLocations.push(location);
      }

      return {
        ...state,
        expandedLocations,
      };
    });
  };
}

function expandRelayLocation(location: RelayLocation): RelayLocation[] {
  const expandedItems: RelayLocation[] = [];

  if ('city' in location) {
    expandedItems.push({ country: location.city[0] });
  } else if ('hostname' in location) {
    expandedItems.push({ country: location.hostname[0] });
    expandedItems.push({ city: [location.hostname[0], location.hostname[1]] });
  }

  return expandedItems;
}

function compareLocationSelectionLoose<SpecialValueType>(
  lhs?: LocationSelection<SpecialValueType>,
  rhs?: LocationSelection<SpecialValueType>,
) {
  if (!lhs || !rhs) {
    return lhs === rhs;
  } else if (lhs.type === LocationSelectionType.relay && rhs.type === LocationSelectionType.relay) {
    return compareRelayLocation(lhs.value, rhs.value);
  } else {
    return lhs.value === rhs.value;
  }
}
