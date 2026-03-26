import { useCallback } from 'react';
import { sprintf } from 'sprintf-js';
import styled from 'styled-components';

import { messages } from '../../../../../../shared/gettext';
import type { CustomListLocation } from '../../../../../features/locations/types';
import { FootnoteMiniSemiBold } from '../../../../../lib/components';
import { FlexColumn } from '../../../../../lib/components/flex-column';
import { spacings } from '../../../../../lib/foundations';
import { Location } from '../location-list-item';
import { useLocationListsContext } from '../location-lists/LocationListsContext';
import { RecentCustomListTrailingActions } from './components';
import { RecentCustomListProvider } from './RecentCustomListLocationContext';

export type RecentCustomListLocationProps = {
  customList: CustomListLocation;
  disabled?: boolean;
};

const StyledLocationContainer = styled.div`
  margin-bottom: ${spacings.tiny};
`;

function RecentCustomListLocationImpl({
  customList,
  disabled: disabledProp,
}: RecentCustomListLocationProps) {
  const { handleSelect } = useLocationListsContext();

  const showEmptySubtitle = customList.locations.length === 0;
  const disabled = customList.disabled || disabledProp;

  const handleClick = useCallback(() => {
    void handleSelect(customList);
  }, [customList, handleSelect]);

  return (
    <StyledLocationContainer>
      <Location root selected={customList.selected}>
        <Location.Accordion expanded disabled={disabled}>
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
            <RecentCustomListTrailingActions customList={customList} />
          </Location.Accordion.Header>
        </Location.Accordion>
      </Location>
    </StyledLocationContainer>
  );
}

export function RecentCustomListLocation({ ...props }: RecentCustomListLocationProps) {
  return (
    <RecentCustomListProvider>
      <RecentCustomListLocationImpl {...props} />
    </RecentCustomListProvider>
  );
}
