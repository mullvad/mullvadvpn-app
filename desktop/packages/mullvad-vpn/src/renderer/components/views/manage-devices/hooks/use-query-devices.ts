import { IDevice } from '../../../../../shared/daemon-rpc-types';
import { useQuery } from '../../../../lib/hooks';
import { useFetchDevices } from './use-fetch-devices';
import { useSortedDevices } from './use-sorted-devices';

export const useQueryDevices = () => {
  const fetchDevices = useFetchDevices();
  const devices = useSortedDevices();
  const { isLoading, isFetching, refetch } = useQuery<IDevice[]>({
    queryFn: fetchDevices,
    queryKey: ['fetch-devices'],
  });

  return { devices, isLoading, isFetching, refetch };
};
