import { ScrollToAnchorId } from '../../shared/ipc-types';
import { ListItemAnimation } from '../lib/components/list-item';
import { useHistory } from '../lib/history';
import { useScrollToReference } from '.';

export const useScrollToListItem = (
  ref?: React.RefObject<HTMLDivElement | null>,
  id?: ScrollToAnchorId,
):
  | {
      animation: ListItemAnimation;
    }
  | undefined => {
  const { location, action } = useHistory();
  const { state } = location;

  const isPop = action === 'POP';
  const anchorId = state?.options?.find((option) => option.type === 'scroll-to-anchor')?.id;
  const scroll = id === anchorId && !isPop;
  useScrollToReference(ref, scroll);

  if (anchorId === undefined || isPop) return undefined;
  return {
    animation: scroll ? 'flash' : 'dim',
  };
};
