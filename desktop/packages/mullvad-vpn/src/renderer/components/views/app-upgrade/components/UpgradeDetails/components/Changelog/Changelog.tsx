import { ChangelogList, NoChangelog } from './components';
import { useShowChangelogList, useShowNoChangelog } from './hooks';

export function Changelog() {
  const showChangelogList = useShowChangelogList();
  const showNoChangelog = useShowNoChangelog();

  return (
    <>
      {showChangelogList && <ChangelogList />}
      {showNoChangelog && <NoChangelog />}
    </>
  );
}
