import { IconBadge } from '../../../../icon-badge';
import { Spinner } from '../../../spinner';
import { useEmptyStateContext } from '../../EmptyStateContext';

export function EmptyStateStatusIcon() {
  const { variant } = useEmptyStateContext();
  return (
    <>
      {variant === 'success' && <IconBadge state={'positive'} />}
      {variant === 'error' && <IconBadge state={'negative'} />}
      {variant === 'loading' && <Spinner size="big" />}
    </>
  );
}
