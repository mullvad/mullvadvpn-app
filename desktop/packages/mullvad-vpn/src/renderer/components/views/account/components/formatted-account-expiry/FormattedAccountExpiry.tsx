import { formatDate, hasExpired } from '../../../../../../shared/account-expiry';
import { messages } from '../../../../../../shared/gettext';
import { Text } from '../../../../../lib/components';

export function FormattedAccountExpiry(props: { expiry?: string; locale: string }) {
  if (props.expiry) {
    if (hasExpired(props.expiry)) {
      return (
        <Text variant="bodySmallSemibold" color="red">
          {messages.pgettext('account-view', 'OUT OF TIME')}
        </Text>
      );
    } else {
      return <Text variant="bodySmallSemibold">{formatDate(props.expiry, props.locale)}</Text>;
    }
  } else {
    return (
      <Text variant="bodySmallSemibold">
        {messages.pgettext('account-view', 'Currently unavailable')}
      </Text>
    );
  }
}
