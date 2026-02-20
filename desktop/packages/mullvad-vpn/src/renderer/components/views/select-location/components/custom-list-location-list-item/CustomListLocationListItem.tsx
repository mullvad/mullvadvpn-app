import { AnimatedList } from '../../../../../lib/components/animated-list';
import { type CustomListLocation } from '../../select-location-types';
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
          return (
            <AnimatedList.Item key={Object.values(child.details).join('-')}>
              <GeographicalLocationListItem
                location={child}
                rootLocation="customList"
                disabled={disabled || loading}
                level={level !== undefined ? level + 1 : undefined}
                position={idx !== customList.locations.length - 1 ? 'middle' : undefined}
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
