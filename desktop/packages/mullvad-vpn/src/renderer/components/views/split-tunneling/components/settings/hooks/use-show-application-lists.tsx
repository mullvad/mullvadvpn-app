import { useShowNonSplitApplicationList } from './use-show-non-split-application-list';
import { useShowSplitApplicationList } from './use-show-split-application-list';

export function useShowApplicationLists() {
  const showNonSplitApplicationList = useShowNonSplitApplicationList();
  const showSplitApplicationList = useShowSplitApplicationList();

  const showApplicationLists = showSplitApplicationList || showNonSplitApplicationList;

  return showApplicationLists;
}
