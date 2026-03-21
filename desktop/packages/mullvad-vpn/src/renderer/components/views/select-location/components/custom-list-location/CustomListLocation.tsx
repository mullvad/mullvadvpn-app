import { useCallback, useEffect, useState } from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { type CustomListLocation } from '../../../../../features/locations/types';
import { FootnoteMiniSemiBold } from '../../../../../lib/components';
import { AnimatedList } from '../../../../../lib/components/animated-list';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { getLocationListItemMapProps } from '../../utils';
import { CustomListLocationListItem } from '../custom-list-location-list-item';
import { Location } from '../location-list-item';
import { useLocationListsContext } from '../location-lists/LocationListsContext';
import { CustomListTrailingActions } from './components';
import { CustomListLocationProvider } from './CustomListLocationContext';

export type CustomListLocationProps = {
  customList: CustomListLocation;
  disabled?: boolean;
};

export function CustomListLocationImpl({
  customList,
  disabled: disabledProp,
}: CustomListLocationProps) {
  const [expanded, setExpanded] = useState(customList.expanded);
  const { handleSelect } = useLocationListsContext();

  const showEmptySubtitle = customList.locations.length === 0;
  const disabled = customList.disabled || disabledProp;

  // Collapse accordion when all its children are removed
  useEffect(() => {
    if (customList.locations.length === 0) {
      setExpanded(false);
    }
  }, [customList.locations.length, setExpanded]);

  // If custom list state is updated from outside, update state accordingly
  useEffect(() => {
    setExpanded(customList.expanded);
  }, [customList.expanded]);

  const handleClick = useCallback(() => {
    void handleSelect(customList);
  }, [customList, handleSelect]);

  const renderChildren = () => {
    return customList.locations.map((locationChild, index) => {
      const { key, nextLevel } = getLocationListItemMapProps(locationChild, 0);
      const position = index !== customList.locations.length - 1 ? 'middle' : undefined;

      return (
        <AnimatedList.Item key={key}>
          <CustomListLocationListItem
            position={position}
            location={locationChild}
            level={nextLevel}
            disabled={disabled}
          />
        </AnimatedList.Item>
      );
    });
  };

  return (
    <Location root selected={customList.selected}>
      <Location.Accordion expanded={expanded} onExpandedChange={setExpanded} disabled={disabled}>
        <Location.Accordion.Header level={0}>
          <Location.Accordion.Header.Trigger
            onClick={handleClick}
            aria-label={sprintf(
              // TRANSLATORS: Accessibility label for a button that connects to a location.
              // TRANSLATORS: Available placeholders:
              // TRANSLATORS: %(location)s - The name of the location that will be connected to when the button is clicked.
              messages.pgettext('accessibility', 'Connect to %(location)s'),
              {
                location: customList.label,
              },
            )}>
            <Location.Accordion.Header.Item>
              <FlexColumn>
                <Location.Accordion.Header.Item.Title>
                  {customList.label}
                </Location.Accordion.Header.Item.Title>
                {showEmptySubtitle && (
                  <FootnoteMiniSemiBold color="whiteAlpha60">
                    {
                      // TRANSLATORS: Label for custom lists that don't have any locations added to them yet.
                      messages.pgettext('select-location-view', 'Empty')
                    }
                  </FootnoteMiniSemiBold>
                )}
              </FlexColumn>
            </Location.Accordion.Header.Item>
          </Location.Accordion.Header.Trigger>
          <CustomListTrailingActions customList={customList} />
        </Location.Accordion.Header>
        <Location.Accordion.Content>
          <AnimatedList>{expanded ? renderChildren() : null}</AnimatedList>
        </Location.Accordion.Content>
      </Location.Accordion>
    </Location>
  );
}
export function CustomListLocation({ ...props }: CustomListLocationProps) {
  return (
    <CustomListLocationProvider>
      <CustomListLocationImpl {...props} />
    </CustomListLocationProvider>
  );
}
