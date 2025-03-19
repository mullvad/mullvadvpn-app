import { ChangelogList, NoChangelog } from './components';
import { useHasChangelog } from './hooks';

export function Changelog() {
  const hasChangelog = useHasChangelog();

  return hasChangelog ? <ChangelogList /> : <NoChangelog />;
}
