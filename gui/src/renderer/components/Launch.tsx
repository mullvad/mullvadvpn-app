import { messages } from '../../shared/gettext';
import { useSelector } from '../redux/store';
import ErrorView from './ErrorView';

export default function Launch() {
  const daemonAllowed = useSelector((state) => state.userInterface.daemonAllowed);
  return (
    <ErrorView showSettingsFooter={daemonAllowed === false}>
      {messages.pgettext('launch-view', 'Connecting to Mullvad system service...')}
    </ErrorView>
  );
}
