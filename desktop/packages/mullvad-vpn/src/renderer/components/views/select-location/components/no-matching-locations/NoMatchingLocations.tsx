import { messages } from '../../../../../../shared/gettext';
import { Flex, Text } from '../../../../../lib/components';

export function NoMatchingLocations() {
  return (
    <Flex
      flexDirection="column"
      alignItems="center"
      padding={{
        top: 'medium',
      }}>
      <Text variant="labelTiny" color="whiteAlpha60">
        {
          // TRANSLATORS: Text describing that the user has applied filters which does not match any servers
          messages.pgettext('select-location-view', 'No matching servers found')
        }
      </Text>
      <Text variant="labelTiny" color="whiteAlpha60">
        {
          // TRANSLATORS: Text describing that the user should try to update their filters
          messages.pgettext('select-location-view', 'Please try changing your filters.')
        }
      </Text>
    </Flex>
  );
}
