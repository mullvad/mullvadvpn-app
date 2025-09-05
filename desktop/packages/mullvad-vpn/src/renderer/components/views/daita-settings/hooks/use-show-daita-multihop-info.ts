import { FeatureIndicator } from '../../../../../shared/daemon-rpc-types';
import { useSelector } from '../../../../redux/store';

export const useShowDaitaMultihopInfo = () => {
  const tunnelState = useSelector((state) => state.connection.status);
  return (
    tunnelState.state === 'connected' &&
    tunnelState.featureIndicators?.includes(FeatureIndicator.daitaMultihop)
  );
};
