import { useCallback, useState } from 'react';
import { sprintf } from 'sprintf-js';

import { messages } from '../../../../../../shared/gettext';
import { CustomListLocation } from '../../../../../features/locations/types';
import { FootnoteMiniSemiBold } from '../../../../../lib/components';
import { AnimatedList } from '../../../../../lib/components/animated-list';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { getLocationListItemMapProps } from '../../utils';
import { CustomListLocationListItem } from '../custom-list-location-list-item';
import { LocationListItem } from '../location-list-item';
import { useLocationListsContext } from '../location-lists/LocationListsContext';
import { CustomListTrailingActions } from './components';
import { CustomListLocationListProvider } from './CustomListLocationListContext';

export type CustomListLocationListProps = {
  customList: CustomListLocation;
  disabled?: boolean;
};

export function CustomListLocationListImpl({
  customList,
  disabled: disabledProp,
}: CustomListLocationListProps) {
  const [expanded, setExpanded] = useState(customList.expanded);
  const { handleSelect } = useLocationListsContext();

  const showEmptySubtitle = customList.locations.length === 0;
  const disabled = customList.disabled || disabledProp;
  const showChildren = customList.expanded || expanded;

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
    <LocationListItem root selected={customList.selected}>
      <LocationListItem.Accordion
        expanded={expanded}
        onExpandedChange={setExpanded}
        disabled={disabled}>
        <LocationListItem.Header level={0}>
          <LocationListItem.HeaderTrigger
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
            <LocationListItem.HeaderItem>
              <FlexColumn>
                <LocationListItem.HeaderTitle>{customList.label}</LocationListItem.HeaderTitle>
                {showEmptySubtitle && (
                  <FootnoteMiniSemiBold color="whiteAlpha60">
                    {
                      // TRANSLATORS: Label for custom lists that don't have any locations added to them yet.
                      messages.pgettext('select-location-view', 'Empty')
                    }
                  </FootnoteMiniSemiBold>
                )}
              </FlexColumn>
            </LocationListItem.HeaderItem>
          </LocationListItem.HeaderTrigger>
          <CustomListTrailingActions customList={customList} />
        </LocationListItem.Header>
        <LocationListItem.AccordionContent>
          <AnimatedList>{showChildren ? renderChildren() : null}</AnimatedList>
        </LocationListItem.AccordionContent>
      </LocationListItem.Accordion>
    </LocationListItem>
  );
}
export function CustomListLocationList({ ...props }: CustomListLocationListProps) {
  return (
    <CustomListLocationListProvider>
      <CustomListLocationListImpl {...props} />
    </CustomListLocationListProvider>
  );
}
