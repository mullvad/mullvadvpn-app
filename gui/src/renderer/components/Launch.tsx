import { messages } from '../../shared/gettext';
import ErrorView from './ErrorView';

export default function Launch() {
  return (
    <ErrorView>
      {messages.pgettext('launch-view', 'Connecting to Mullvad system service...')}
    </ErrorView>
  );
}
