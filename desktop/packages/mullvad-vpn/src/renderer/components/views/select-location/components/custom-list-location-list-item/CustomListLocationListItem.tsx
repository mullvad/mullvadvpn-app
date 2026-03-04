import { type CustomListLocation } from '../../../../../features/locations/types';
import { AnimatedList } from '../../../../../lib/components/animated-list';
import { getLocationListItemMapProps } from '../../utils';
import { AnyLocationListItem, type AnyLocationListItemProps } from '../any-location-list-item';
import { GeographicalLocationListItem } from '../geographical-location-list-item';
import {
  CustomListLocationListItemProvider,
  useCustomListLocationListItemContext,
} from './CustomListLocationListItemContext';
import { useHandleSelectCustomList } from './hooks';

export type CustomListLocationListItemProps = Omit<
  AnyLocationListItemProps,
  'location' | 'onSelect'
> & {
  disabled?: boolean;
  customList: CustomListLocation;
};

function CustomListLocationListItemImpl({
  customList,
  level,
  disabled,
  ...props
}: CustomListLocationListItemProps) {
  const { loading } = useCustomListLocationListItemContext();
  const handleSelectCustomList = useHandleSelectCustomList();

  return (
    <AnyLocationListItem
      location={customList}
      rootLocation="customList"
      level={level}
      disabled={disabled || loading}
      onSelect={handleSelectCustomList}
      {...props}>
      <AnimatedList>
        {customList.locations.map((child, idx) => {
          const { key, nextLevel } = getLocationListItemMapProps(child, level);

          // Since list item is wrapped with animated list item, we need to manually
          // tell it what position it is in the list.
          const position = idx !== customList.locations.length - 1 ? 'middle' : undefined;
          return (
            <AnimatedList.Item key={key}>
              <GeographicalLocationListItem
                location={child}
                rootLocation="customList"
                disabled={disabled || loading}
                level={nextLevel}
                position={position}
                onSelect={handleSelectCustomList}
                {...props}
              />
            </AnimatedList.Item>
          );
        })}
      </AnimatedList>
    </AnyLocationListItem>
  );
}

export function CustomListLocationListItem({ ...props }: CustomListLocationListItemProps) {
  return (
    <CustomListLocationListItemProvider>
      <CustomListLocationListItemImpl {...props} />
    </CustomListLocationListItemProvider>
  );
}
