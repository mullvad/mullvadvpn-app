import { messages } from '../../../../../../shared/gettext';
import { FootnoteMini, Icon } from '../../../../../lib/components';
import { Flex } from '../../../../../lib/components/flex';

export function AutomaticLocationOverridesFiltersInfo() {
  return (
    <Flex gap="tiny">
      <Icon size="tiny" color="whiteAlpha60" icon="info-circle" />
      <FootnoteMini color="whiteAlpha60">
        {
          // TRANSLATORS: Info text describing that filters are not applied when an automatic location is used.
          messages.pgettext(
            'filter-view',
            'Filters are overridden when using an automatic location',
          )
        }
      </FootnoteMini>
    </Flex>
  );
}
